# Exploring digital twins with Raspberry Pi Pico

«Digital Twin» is having a digital representation of a physical device, which you can interact with just like the physical device. What does this mean, and why is it useful? Read on to see how you can make a digital representation of the Raspberry Pi Pico W.

## Background

A digital twin has many use cases, but in this article, we will focus on interfacing with and reacting to events from an existing physical device. We've also [written about digital twins](https://blog.drogue.io/digital-twins/) before, in the context of Eclipse IoT.

Why is this useful? Once you've written a few IoT applications, a pattern emerges, and there are a few operations that many applications follow:

- Read the current state reported a device.
- Trigger some action based on the state or some threshold.
- Send a command back to the device

That's quite simple, why do you need a digital twin to do that? Consider the following:

- Your device state most likely follow some kind of schema, which you may (or may not) want to enforce.
- You probably need to persist the device state, in case the system goes down and your device reports it's state every hour.
- Running an action based on a state update in real time per device must scale to many devices.
- The command to a device may get lost on the way, using a messaging system will not help you here.

A digital twin system handles the above for you, using mechanisms that works for _most of the use cases_. For the other use cases, you can still tap into the telemetry stream.

For this article, we'll use [Drogue DoppleGänger](https://book.drogue.io/drogue-doppelgaenger/dev/index.html), which you can read about in the [previous blog post](https://blog.drogue.io/drogue-cloud-zero-eleven/).

## Hardware

In this setup we'll use the [Raspberry Pi Pico W](https://www.raspberrypi.com/documentation/microcontrollers/raspberry-pi-pico.html), which you can get for $6. The board has a WiFi peripheral, which means we can talk directly to Drogue Cloud.

## Software

We'll be using the public [Drogue Cloud Sandbox](https://sandbox.drogue.cloud/) service, and run [Drogue DoppleGänger](https://book.drogue.io/drogue-doppelgaenger/dev/index.html) locally using Podman (also works with Docker).

For the device, we'll use [this example](), which already works with Drogue Cloud.


## Configuring

To prepare using the Digital Twin, we must create and configure resources in Drogue Cloud (replace `twin-app` with an app name of your choice):

```
drg login https://api.sandbox.drogue.cloud
drg create app twin-app
drg create device pico --app twin-app
drg set password pico its-me --app twin-app
```
