+++
title = "Drogue Cloud: Release 0.9.0"
extra.author = "ctron"
description = "Drogue Cloud 0.9.0 comes with a bunch of new features, and finally cleans up two major nuisances."
+++

A lot has happened since the last release of Drogue Cloud. Unfortunately, the only thing that seems to stick out
is the ongoing war in Ukraine. And whatever you think might have led to this, there is never, ever a justification
for bombing the civilian population of a country. So, in a situation like that, can we simply celebrate what we achieved
in Drogue IoT, like we usually do? I believe the answer to that must be "yes". Do what you think is right, but also
focus on the positive! Because otherwise the terrorists have won!

<!-- more -->

## Application monitoring: Metrics and Tracing

I am not sure if I need to advertise the benefits of application monitoring. We wanted to have that for a while. Not only
as a feature, but actually for using that ourselves in our public sandbox. And, as we have been testing performance and
throughput, having metrics and tracing comes in handy too.

Drogue Cloud 0.9 exposes metrics using a Prometheus endpoint and supports tracing with Jaeger tracing out of the box.
When it comes to tracing, we not only support Jaeger, but we are tapping into the Rust ecosystem of
[tracing](https://tracing.rs/tracing/) and [opentelemetry](https://docs.rs/opentelemetry/latest/opentelemetry/).
However, that doesn't come out-of-the-box.

## Digital twin: Early Eclipse Ditto™ integration

Drogue Cloud's current focus is on IoT connectivity. But we definitely want to have an (optional)
integration with a digital twin component too. So, why not go ahead and bring together Drogue Cloud and
[Eclipse Ditto](https://eclipse.org/ditto/)?

Eclipse Ditto is an open source digital twin solution, and sounds like a perfect fit. Bringing those two together took
a bit of an effort, and isn't fully finished yet. This will be a topic, that for sure spans over multiple releases.
However, a lot of things are already in good shape, and I would also encourage you to try it out. The existing
digital twin Helm charts have been adapted to install the integration if you opt-in for that. When you installed it,
you can:

* Enable an application in Drogue Cloud for Ditto
  * This will set up the connection between Drogue Cloud and Ditto
  * And create the necessary policies
* Enable devices in Drogue Cloud to be mapped to Eclipse Ditto
  * Which creates matching "things" in Ditto
  * And applies the correct policy
* Delete resources on the Drogue Cloud side, which will synchronize this with the Ditto side

You can track the remaining items here: [drogue-iot/drogue-cloud#223](https://github.com/drogue-iot/drogue-cloud/issues/223)

Is Eclipse Ditto the only thing that we have in mind? No. Ditto is a powerful beast. But it also has quite an appetite
for resources. Not everyone might need all the bells and whistles, so we are still investigating other options too. In
the end, there is no plan for have a single solution, but to allow integrating with more than one.

## Updating dependencies

Sounds boring? True! However, this time, we were a bit excited about that. To understand this, let's have a closer look
into Rust, its dependencies, and Tokio.

Rust follows a [semantic versioning](https://semver.org/) scheme. And a core dependency of many Rust applications that
deal with I/O and need an asynchronous executor, is [tokio](https://tokio.rs/). Now, for a long time, tokio was released
as `0.2`, which was the basis for a lot of other libraries.

At some point tokio `1.0` was released, with great joy. Now the problem is that tokio 0.2 and 1.0 are not compatible
by definition. Also, due to the fact that Tokio provides an asynchronous executor, you cannot just mix 0.2 and 1.0
in a single application (that easily). On the bright side, the migration wasn't that difficult, and so may libraries
did this, and released new versions of themselves too.

With one exception, that was [actix](https://actix.rs/). We use actix for HTTP endpoints, and there are a bunch of
extensions on top of actix (like metrics, CORS, OAuth2), which make your life a lot easier. On the other side, e.g. the
asynchronous PostgreSQL driver was a thing we didn't want to miss. So we carefully curated our dependencies until
there was a beta version of actix-web, which supported tokio 1.0. So we migrated to that in the hope that, like most
other libraries, there would be a release soon, which would adopt tokio 1.0.

That was several Drogue Cloud releases back. Actix decided to do all kind of breaking changes too, because in the end,
the migration to tokio 1.0 also was a breaking change.

A few weeks back, actix-web 4 was finally released. And with that, most on-top features, like the Cloud Events SDK, too.
Finally, we could drop a lot of carefully curated "beta" dependencies and git patches from our `Cargo.toml` files.

Sure, we also updated Strimzi, Keycloak, and a bunch of other dependencies. But none of these felt as good as ripping
off all the duct tape.

## Improve frontend build

Another dependency we updated was [Yew](https://yew.rs/). Yew is the web frontend framework we use for the Web Assembly
based console we have. We skipped one release (0.18), because we knew that the next release (0.19) would break things
again. So migrating from 0.17 to 0.19 wasn't a simple task, but it also was the right time to get rid of another
nuisance: Webpack!

If you live in the JavaScript world, you might know webpack. It is the go-to tool for compiling web frontend
application and bundling it up. If you have a JavaScript allergy, like I have, you might have a hard time with that
tool. Especially, as wasm and webpack don't really play well together.

In the Rust world however, there is a new tool available: [trunk](https://trunkrs.dev/). Trunk does the same job that
webpack does. Just in a much saner way (and webpack fans might disagree here). For us that means that from a change
to a hot-reload in the console it is only a few seconds, instead of more than two minutes. And it doesn't silently break
the console when doing a dependency upgrade on the tool.

That will also allow us to improve the developer experience on the web console, and we should see some nice improvements
and fixes in our console, in the next releases.

## MQTT over websocket

We had MQTT, even 3.1.1 and 5. What was missing, was MQTT over web sockets. Thanks to
[ntex](https://github.com/ntex-rs/ntex), that is now fixed! You can directly send data over MQTT/WS from your browser.

But you can also use MQTT over websocket to connect to the integration endpoint, and consume data from Drogue Cloud.
You could do that before too, using the web socket integration, but it wasn't a huge effort, and fans of MQTT might
actually like this.

## MQTT dialects

Drogue Cloud is not a general purpose MQTT broker. However, we do offer an API on top of MQTT to interact with
Drogue Cloud. That is pretty straight forward. But still, some definitions on how do use MQTT exist. We always had the
plan to extend this, and allow for other APIs and opinions, but didn't have any reason to do so.

Until we got approached by a member of the community, asking for a different topic pattern than what Drogue Cloud
currently offers. After a bit of discussion, we even got a PR, implementing part of the change. Many thanks to Julian
from [pragmatic minds](https://pragmaticminds.de/) for pursuing this.

So in Drogue Cloud 0.9, it is now possible to configure the MQTT dialect on a per-application level, and override it on
a per-device level. Right now, we have two dialects: the original, Drogue Cloud one. And a "plain topic one". Not too
much of a difference, but that is what made sense to Julian.

## Event processing

When events flow into the system, it sometimes makes sense to do some early processing. For the Ditto integration
for example, we had to replace the content type, to identify Ditto protocol payload. As you can't explicitly provide
a "content type" with MQTT 3.1.1, we needed an alternative.

We also wanted to classify values with an AI/ML model, directly when consuming events from devices. So that the
consuming application have some AI/ML enriched data.

With Drogue Cloud 0.9, it is now possible to define some inbound rules, which allow stateless processing of events as
they come in. And just to be clear, that isn't a complex rule based engine, but an opinionated, lightweight feature

One might ask why it makes sense to have this as part of Drogue Cloud? Why not add something like Camel or Drools?
Or send everything to an external endpoint? The answer is simple: We only support extremely lightweight operations!
Like changing a header value. So it doesn't make sense to spin up a pod just for that. And by limiting the operations
we support, we can ensure that no one is mining bitcoins on our infrastructure.

And if that isn't enough for your use case, then we do support handing off that operation to an external endpoint which
you can provide. Using Camel or Knative with that is definitely an option. Here are the operations we currently
support as part of Drogue Cloud:

* Activate based on conditions, like cloud event attributes
* Modifying cloud event attributes
* Contacting an external endpoint to validate the event
* Contacting an external endpoint to enrich/modify the event

There are a lot more operations we could add. But again, we don't want to re-invent anything like Camel. However, if it
is super lightweight, doesn't include user code, and might be useful for a lot of people, we could definitely add it.

## Out-of-the-box Knative event source

Events that come in to Drogue Cloud can be consumed out of Drogue Cloud directly using Kafka, or via the MQTT and
WebSocket integration. All of those interfaces are consumer initiated variants. The reason for that is simple, as long
as no one is interested in consuming the data, no processing is performed and no resources are consumed in the cloud
side. Once a consumer is interested, it will establish a connection to Drogue Cloud, and starts to consume the data.

However, when you want to create a serverless processing of data, that model doesn't work well. Assuming you have a
Knative serving endpoint, which waits for IoT to be sent, a model which has to initiate the communication doesn't work.
The idea of Knative serving is to scale down to zero if no data is being sent, and scale up as required to be able
to handle all requests.

With Drogue Cloud 0.9 we added a producer side initiate model. You can configure a Drogue Cloud application to try and
deliver messages from the event stream to a Knative serving endpoint. It doesn't need to be run by Knative, it simply
is an HTTP, which will receive Cloud Events from Drogue Cloud. Once configured, Drogue Cloud will spin up a pod and
start delivering data to the endpoint you provided. Thanks to Rust, this only requires a few MiB of RAM on our side.

## What's next?

With all the new features, we definitely want to create some more workshops to show you how it works. We are already
working on something that leverages the Knative event source. That might be really cool.

Having upgraded Yew and streamlined the web development workflow, the console will also receive a bit more attention.
There is a lot more we could do, we know that.

Having metrics and tracing available, we already did take a quick look at scale testing. This is definitely something
that we want to focus on more in this year, but also the first impression was quite encouraging. But we also know that
we could add much more detail to metrics and tracing. Like the epic of digital twins, this item will stay on our focus
for a while longer.

I do hope that sanity prevails, and that we can look forward to a much brighter summer release. Stay safe!

## Also see

* [Releases](https://github.com/drogue-iot/drogue-cloud/releases)
* [Public sandbox](https://sandbox.drogue.cloud)
