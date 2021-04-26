+++
title = "Rebasing Drogue Device"
extra.author = "lulf"
+++

For the past few weeks, we explored removing the allocator from drogue-device and adopt drivers to a more restrictive Actor model. Read on to learn why and how drogue device will change, and the benefits of this change.

<!-- more -->

# Background

After the initial [announcement](https://blog.drogue.io/introducing-drogue-device/) of drogue-device, we've expanded the set of drivers that cover [WiFi](https://github.com/drogue-iot/drogue-device/tree/main/examples/stm32l4/iot01a) and [LoRa](https://github.com/drogue-iot/drogue-device/tree/main/examples/stm32l0/lora-discovery), and have demonstrated that drogue-device can [talk to drogue-cloud](https://blog.drogue.io/drogue-device-to-cloud/). The actor model as improved after feedback from the Rust community, and we think that writing firmware composed of `Actors` is a nice way to program. 

At the same time, we sometimes touched on a issues related to memory management and missing async features in Rust, so we felt that the time was right to evaluate our approach so far and look at some of the challenges:

* Although Rust async is great, there are some features that makes async harder on embedded.
* The actor model allows implementing handlers for arbitrary messages for an actor. This flexibility of the current actor model requires a static allocator, and in turn prevents knowing memory usage of an application upfront.
* A driver package could end up using 2-3 actors, thereby significantly increasing the memory footprint of an application, causing issues when doing encryption and things that have a high peak memory use.
* With a goal of developing a bootloader and supporting firmware updates from drogue-cloud, and working with TLS, we needed to reduce the footprint.

For the past few weeks, we have been working on a way to remove the static allocator from drogue-device, and re-use the ongoing [async embedded](https://github.com/embassy-rs/embassy) effort driven by the community.

But first, lets have a look at what a simple drogue-device application with a single "counter actor" looks like today:

```rust
struct CounterActor {
    counter: u32
}

struct Increment;
struct GetCount;


impl Actor for CounterActor {
    type Configuration = ();
    fn on_start(self) -> Completion<Self> {
        Completion::defer(async move {
            self.counter = 0;
        });
    }
}

impl NotifyHandler<Increment> for CounterActor {
    fn on_notify(mut self, message: Increment) -> Completion<Self> {
        Completion::defer(async move {
            self.counter += 1;
            self
        });
    }
}

impl RequestHandler<GetCount> for CounterActor {
    type Response = u32;
    fn on_request(mut self, message: GetCount) -> Response<Self::Response> {
        Request::defer(async move {
            (self, self.counter)
        });
    }
}

struct MyDevice {
    counter: ActorContext<CounterActor>,
}

impl Device for MyDevice {
    fn mount(&'static self, _: DeviceConfiguration<Self>, supervisor: &mut Supervisor) {
        let address = self.counter.mount((), supervisor);
        address.notify(Increment);
        // Pass address of to some other actor
    }
}

fn configure() -> MyDevice {
    MyDevice {
        counter: ActorContext::new(CounterActor{ counter: 0 });
    }
}

#[entry]
fn main() -> ! {
    device!(MyDevice = configure; 1024);
}
```

To keep the example small, additional actors that use the counter are left out. 

At the core is the Device, which holds on to a set of actors. An actor implements the `Actor` trait, and for every message type it handles, it implements a `NotifyHandler` or a `RequestHandler`. Messages are sent using an `Address` handle, that is produced when mounting an actor.

For further introductions to drogue-device, have a look at our [book](https://book.drogue.io/drogue-device/dev/concepts.html).

So, lets talk about the problems and how that impacts the framework.

## Problem 1: what type of message can an `Actor` handle?

In drogue-device, an actor may handle different types of messages. To handle a new message type, a `RequestHandler` or `NotifyHandler` trait implementation for the message type is written.

Each actor in drogue-device is accompanied by a `ActorContext`, which owns the Actor and, amongst other things, a queue for the incoming messages.

What type is the element in the queue? Because an `Actor` can have a `RequestHandler` implemented for any Rust type, it can be any type! Therefore, we cannot know at compile time the size of the queue. In practice this means that the message must be stored on a heap, so alloc and `Box` must be used in order to get a fixed size element.

Solution:

To avoid alloc, the `ActorContext` must know the type of messages beforehand. This requires putting some restrictions on an `Actor`: it must only handle messages of a single type. So, instead of writing a `RequestHandler` for different message types, the `Actor` trait tself defines an associated type `Message`, which the implementor uses to specify the message type. In addition, the message handler function signature for that type is part of the `Actor` trait.

In short, instead of this:

```rust
trait Actor { }

trait RequestHandler<M> {
    fn on_request(&self, message: M);
}
```

The `RequestHandler` trait is removed, and the `Actor` trait is modified to this:

```rust
trait Actor {
    type Message;
    fn on_message(&self, message: Self::Message);
}
```

But, all is not lost: To handle multiple message types, an enum can be used, and the implementation can use pattern matching to perform the appropriate action. The end result is that the size of incoming message queue is known at compile time, yay!


## Problem 2: what is the size of an async function?

The second problem was more difficult, and relates to how async-await works in Rust. So, first a little side-track.

### Futures in Rust

The async-await syntax is quite compact:


```rust
async fn double_it(arg: u32) -> u32 {
    arg * 2
}
```

This function get translated to a 'unit of code' that can be executed (polled) later, aka. a `Future`. The way Rust does that is to generate a type that implements the `Future` trait. The future trait has a method that allows you to `poll` the future, which will drive it to completion, or allow you to register a `Waker` used to signal the runtime that the future can be polled again.

To run the future, a runtime also called an  `executor` is used, and there are several of those out there. Drogue-device have had its own executor that runs actors, and each actor may have one "current" future that can be polled. The implementation simply iterates over all the actors, ask them if they have any futures stored that it should poll, and then polls them. 


There are many resources on this topic, such as the [async book](https://rust-lang.github.io/async-book/02_execution/01_chapter.html), or this excellent blog post on [pin and suffering](https://fasterthanli.me/articles/pin-and-suffering).

### Back to the problem

Well, the problem relates to the _size_ of a future. When drogue device calls an `Actor`s `RequestHandler`for a given message, the `RequestHandler` returns a `Future`! However, the _size_ of that `Future` depends on the code within it (so, it can know how much stack memory is needed by code).

If you want to store a future (on the `ActorContext`) to be polled at some later point by the executor, how do you do that? By using alloc and putting the future in a `Box`!

In drogue device this can be seen in the signature of `RequestHandler`, which returns a `Response`. What this does behind the scene is to use alloc and store the future in a `Box` so that it can be stored on the `ActorContext`. 

Solution:

To resolve this we need to constrain the `Actor` even more: For each `Actor` implementation, we must know the size of the future it return. This, in turn, means that it can only return a "known" future that the compiler can understand the size of.

In rust stable, this means that all actors have to return something that implements the `Future` trait, and that just makes writing Actors too hard.

The ideal feature needed to handle this is really "Async Traits" - being able to define async functions in traits (the `Actor` trait) and then have the compiler magically figure out which implementation is used and calculate the known size. 

Unfortunately, this will not come to Rust for some time. In Rust nightly, however, there are a few compiler features we can enable, that gives us the ability to use nice async {} blocks in the `Actor` implementations:

```rust
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]
```

Combined, these allow us to use associated types in traits with lifetimes, in combination with `impl Future` as a type. 

To sum it up, instead of defining the trait like this and requiring alloc:

```rust
trait Actor {
    fn do_something(&self) -> Box<dyn Future>;
}
```

One can do this instead:

```rust
trait Actor {
    type TheFuture<'m>: Future<Output = ()> + 'm;
    fn do_something<'m>(&self) -> Self::TheFuture<'m>;
}
```

And the resulting implementation can look like this:

```
impl Actor for MyActor {
    type TheFuture<'m> = impl Future<Output = ()> + 'm;
    fn do_something<'m>(&self) -> Self::TheFuture<'m> {
        async move {
            // Do some cool async-await stuff!
        }
    }
}
```

Ok, so thats not too bad. It's still a bit away from fully async traits, but it will have to do.

## Drogue-device reborn

With these things sorted out, we are able to remove the static allocator, at the cost of requiring Rust nightly features. 

Another project doing async rust embedded that were already going for Rust nightly was the [Embassy Project](https://github.com/embassy-rs/embassy). 

If we were anyway going to use nightly, what if we reused the executor and HAL from Embassy?

# Embassy

Embassy is a project to make async/await a first-class option for embedded development. All the way from the Hardware Abstraction Layers (HAL), to the executor and running tasks. 

All interaction with peripherals can be done using async-await, which is a great model to work with when dealing with timers, interrupts and so on. One can write code like the following to wait for an interrupt with a timeout:

```rust
let interrupt_fut = self.irq.wait_for_rising_edge();
let timeout_fut = Timer::after(timeout);

match select(interrupt_fut, timeout_fut).await {
    Either::Left((r, _)) => // Handle interrupt
    Either::Right(_) => // Handle timeout
}
```

After playing around with Embassy and, we decided that using embassy as the foundation of our acter model was a viable approach.

# Drogue-device rebased

Having modified drogue-device actor model and rebased it on embassy, we see the following improvements:

* Flash usage reduced by 2x
* Static RAM usage reduced by ~7x

We also have good reasons to believe stack usage is somewhat reduced by only using Actors for the cases where shared access to some resource, or the ease of composition is desired.

In addition, there are additional benefits like running drogue-device the host, which can simplify driver development.

# So what does it look like?

So, lets take a look at what the example at the beginning of this post will look like when rewritten to the new world:

```rust
struct CounterActor {
    counter: u32
}

enum CounterRequest {
    Increment,
    GetCount(u32),
}

impl Actor for CounterActor {
    type Configuration = ();
    type Message<'m> = CounterRequest;

    type StartFuture<'m> = impl Future<Output = ()> + 'm;
    fn on_start<'m>(&mut self) -> Self::StartFuture<'m> {
        async move {
            self.counter = 0;
        }
    }

    type MessageFuture<'m> = impl Future<Output = ()> + 'm;
    fn on_message(&mut self, message: Self::Message<'m>) -> Self::MessageFuture<'m> {
        async move {
            match message {
                CounterRequest::Increment => self.counter += 1,
                CounterRequest::GetCount(c) => *c = self.counter,
            }
        }
    }
}

#[derive(Device)]
struct MyDevice {
    counter: ActorContext<'static, CounterActor>,
}

#[drogue::main]
async fn main(mut context: DeviceContext<MyDevice>) {
    context.configure(MyDevice {
        counter: ActorContext::new(CounterActor { counter: 0 }),
    });
    
    let address = context.mount(|device| {
        device.counter.mount(())
    });
    address.notify(CounterRequest::Increment).await;
}
```

Notice that the main function is now fully async, which in turn simplifies the rest of the work around configuring the device, mounting the actors and from then using the addresses to send messages to actors.

You can also see the restrictions imposed in order to allow zero alloc: a single message type per actor, and using associated types to define the futures that are returned by the actor.


Unit testing is also simplified:

```
#[drogue::test]
async fn mytest(context: TestContext<MyDevice>) {
    // Do the stuff you'd normally do in main + assertions
}
```

The `TestContext` has the same API as the `DeviceContext`, extended with methods to create async signals, dummy pins and a controlled shutdown of the test.

# Whats next?

At the time of writing, all the work is done in a [separate repository](https://github.com/drogue-iot/drogue-device-ng), with the goal of replacing the existing drogue-device repository once most of the remaining drivers and examples have been moved over. If you want to contribute to this effort, reach out in the drogue iot chat.

# Summary

For the past few weeks, we explored removing the static allocator from drogue-device and adopt drivers to a more restrictive Actor model. After several attempts, we could not find a way to do this without starting to use features from Rust nightly. Having moved to nightly, the barrier for adopting an existing framework like embassy as the foundation was lower. And the outcome have been all positive. The Embassy project have been very helpful in answering questions, discussing our problems and reviewing patches that we've submitted.
