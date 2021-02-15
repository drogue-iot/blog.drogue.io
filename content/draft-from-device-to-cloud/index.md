+++
title = "From drogue-device to drogue-cloud"
extra.author = "ulf_and_jens"
+++

In previous posts we've seen how to [run drogue-cloud](https://blog.drogue.io/the-cloud-side-of-things/), how to [use LoRaWAN in rust](https://blog.drogue.io/rust-and-lora/), and the introduction of the [drogue-device](https://blog.drogue.io/introducing-drogue-device/). In this post, we'll tie this together and walk through the process of running LoRa on drogue-device, sending data to drogue-cloud. 

<!-- more -->

# Recap

* About drogue-cloud

## About LoRa

LoRa is a low power long range wireless protocol that operates in a lower frequency spectrum than WiFi, ZigBee and Bluetooth. This enables IoT use cases not possible with the shorter range technologies. We've [previously](https://blog.drogue.io/rust-and-lora/) seen how you can use LoRa with Rust. In this post, LoRa is only used as an example of one way to communicate from a drogue-device to drouge-cloud. We aim to support both WiFi, LoRa, NB-IoT and other wireless standards for working with drogue-cloud.

## About drogue-device

Drogue-device is an [Actor framework](https://en.wikipedia.org/wiki/Actor_model) for writing embedded applications in Rust. The advantage of using [drogue-device](https://github.com/drogue-iot/drogue-device) is that you can represent sensors and peripherals as independent components (actors) and wire them together in your application, as a way to apply good software engineering principles in embedded programming.

# Device drivers on drogue-device

Drogue-device contains device drivers for different boards and sensors. Drivers follow a common set of patterns that makes it easier to write new drivers. Device drivers can be written in different ways. The common patterns we've seen is:

* Writing a generic driver that consists of a hardware specific HAL that implements a HAL trait, and an actor that is templatized with the HAL trait as a type parameter. The actor is the API outwards to the embedded application and the drogue-device framework. 

* Writing a common set of commands that an Actor should support, and writing a hardware-specific driver that implements the request and notify handlers for the commands. This may be easier if a lot of the driver logic is hardware-specific, and there would be little gain in using a common HAL.

In both cases, a `Package` may be used to group multiple actors in a driver together and expose a single primary actor used to interact with the device.

For more information, see [the driver guide](https://github.com/drogue-iot/drogue-device/blob/master/DRIVERS.md).

In this example, we'll be using the [Rak811]() driver for LoRa, and the [nRF UART]() driver. These are independent drivers, that are wired together using message passing. All interactions with the peripheral is done using the drivers _address_ that is configured during initialization.

To send a message, the LoRa driver API is used:

```rust
   lora_driver_address.send(QoS::Confirmed, 1, b"Hello").await.expect("Error sending data");
```

The full example of drogue-device can be found [here](https://github.com/drogue-iot/drogue-device/tree/master/examples/nrf/microbit-rak811).


# Telemetry to the cloud

* Sending data to TTN
* Why TTN and not drogue-cloud directly?
* How it could work using drogue-cloud directly

# Preparing drogue-cloud

* Introducing drogue-cloud sandbox
* Creating application
* Device management

# Integrating TTN with drogue-cloud

* Image + description of TTN console showing the integration
* Setting up auth

# Sending the data

* Opening drogue sandbox console
* Interacting with the device
* View incoming data

# Future work

* Summarize
* Device management integration
* Footprint of drogue-device
* Firmware upgrades
