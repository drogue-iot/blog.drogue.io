+++
title = "Firmware updates, part 3: Firmware Delivery and Building"
extra.author = "lulf"
+++

This is the third post in a series about doing device firmware updates (DFU) over the air (OTA) and continuous deployment of firmware for embedded devices. We'll explore the different parts of a complete end-to-end system with this capability.

This post is about how the firmware is delivered to devices and how you can build the firmware using Drogue IoT.

<!-- more -->

# Background

In the [previous post](https://blog.drogue.io/firmware-updates-part-2/), we explored the ways which you can transport the firmware onto devices across different transports.

Now we're going to look at the final steps: building and delivering the firmware, based on the update protocol defined by [`embedded-update`](https://github.com/drogue-iot/embedded-update). First, let's keep in mind what types of devices we're updating:

* No operating system
* Limited/reduced network connectivity
* Small amounts of RAM

For devices that are not as light weight and can run Linux, there are other options such as [Flotta](https://project-flotta.io/) that might be a better fit.

Supporting firmware updates in Drogue IoT is something we've been thinking about for a while, since it's a common piece of any IoT infrastructure. There are many ways to distribute software, but there are a few important properties when dealing with tiny IoT devices:

* Connectivity - embedded devices may not be connected at all times and not able to retrieve the entire firmware in one batch
* Protocol support - embedded devices may use a wide rang of different protocols as discussed in the [previous post](https://blog.drogue.io/firmware-updates-part-2).
* Footprint - memory on devices and bandwidth may be limited, therefore the protocol must impose minimal overhead

Our first attempt was to offload this functionality to an existing project like [Eclipse Hawkbit](https://www.eclipse.org/hawkBit). The Eclipse Hawkbit update protocol is stateless (server processes requests independently) and allows devices to fetch firmware in chunks as requested by device. One downside is that Eclipse Hawkbit update service only supports HTTP with a JSON protocol, not really suitable for tiny devices. Moreover, it requires maintaining the firmware state of a device separately from the device management in Drogue Cloud.

With the telemetry and command API of Drogue Cloud, we have all the building blocks needed to transport the firmware. So instead of using Hawkbit directly, we can define an update protocol using Drogue Cloud connectivity as a transport and use the Drogue Cloud integration APIs for our update service. This unlocks some interesting use cases:

* The service can run in a private network not exposed to the public
* The service can integrate with different backends, such as Eclipse Hawkbit
* The service can provide updates over any protocol supported by Drogue Cloud

Another lesson in our experimentation with Rust on embedded devices, is that writing Rust feels like writing software running on a server. This had some implications on how we thought about software for embedded: what if we could apply the same mechanisms and tooling we use for building "normal" software to firmware for embedded devices? After all, projects like [Tekton](https://tekton.dev) allow you to define a CI/CD pipeline that builds any software using container images on Kubernetes. Moreover, container registries are used to store and retrieve software, so maybe we could store firmware in those registries as well?

The end result after exploring the above ideas is an add-on to Drogue Cloud named Drogue Ajour (à jour => updated).

To learn more about Drogue Ajour than what is covered by this article, have a look at [the documentation](https://book.drogue.io/drogue-ajour/dev/index.html). As with all things in Drogue IoT, the code is open source, and available [on github](https://github.com/drogue-iot/drogue-ajour).

## Drogue Ajour

Drogue Ajour is a firmware update and build service for tiny devices connected to Drogue IoT Cloud. It supports a wide range of IoT protocols and uses a low footprint update protocol.

<figure>
    <img src="ajour1.png" alt="Drogue Ajour Console - Overview" />
    <figcaption>Drogue Ajour Console - Overview</figcaption>
</figure>

It offers:

 * Update service - delivering firmware updates to devices connected to Drogue Cloud
 * Protocol support - any protocol supported by Drogue Cloud (HTTP, MQTT, CoAP, LoRaWAN)
 * Firmware build - building firmware and storing it in a firmware repository.
    * RESTful API for inspecting and triggering builds
 * Management console to inspect firmware build and roll-out status for all your Drogue Cloud devices

It is built on top of:

 * [_Drogue Cloud_](https://drogue.io) - For authentication and the connectivity layer for devices
 * [_Tekton_](https://tekton.dev) - For defining a CI/CD pipeline for firmware builds

You can run Drogue Ajour locally or on a Kubernetes cluster. With it, you can build a firmware delivery pipeline for your devices in Drogue Cloud.

### Overview

Drogue Ajour is composed of 2 main components serving different functions:

* _Firmware Delivery_ - Transporting firmware updates to devices
* _Firmware Build_ - Building and storing firmware artifacts

Of these, only the firmware delivery component is mandatory.

<figure>
    <img src="architecture.png" alt="Drogue Ajour Architecture" />
    <figcaption>Drogue Ajour Architecture</figcaption>
</figure>

## Firmware delivery

Firmware delivery is the main functionality of Drogue Ajour. This involves transporting the firmware to devices using a CBOR-based protocol (Concise Binary Object Representation). Although more compact formats than CBOR exists, it is an IETF standard as well as having implementations in [many languages](https://cbor.io/impls.html). The CBOR messages are designed for minimal overhead, and to allow devices to consume updates at their own pace.

The update service can use the file system, Eclipse Hawkbit or a Docker/Container registry for retrieving firmware.

To enable firmware updates, the Drogue Cloud `Application` and `Device` schema is extended with a section for specifying the firmware source, which looks like this for a container registry:

```
spec:
    firmware:
        container:
            image: my-firmware:latest
```

The container image reference is relative to the container registry configured for the Drogue Ajour instance. A device reports its firmware status periodically to Drogue Ajour using the 'dfu' Drogue Cloud channel. The update service will look for messages on this channel, and compare the status with the latest firmware stored in the image. If needed, it will proceed by sending updates back to the device using the Drogue Cloud command API. If the device is still waiting for a command (usually within a timeout), or is connecting at a later time, it will receive the firmware data corresponding to it's status message.

<figure>
    <img src="ajour4.png" alt="Drogue Ajour Console - Device View" />
    <figcaption>Drogue Ajour Console - Device View</figcaption>
</figure>


The update protocol is stateless, meaning that Drogue Ajour will track only send out firmware updates to each individual status update reported by devices. NOTE: In the event that devices are not awaiting command messages with a sufficiently long timeout, duplicate messages may occur, so the timeout is an important consideration to avoid that depending on how the device is connected.

## Firmware build

This is an optional component that allow you to build your firmware from source and make it available to the delivery component for rolling out to your devices (aka Source-To-Firmware).

Drogue Ajour provides Tekton pipeline definitions that can build firmware images in the expected format and push them to container registries.

<figure>
    <img src="ajour2.png" alt="Drogue Ajour Console - Build View" />
    <figcaption>Drogue Ajour Console - Build View</figcaption>
</figure>

### Build specification

The specification put on the `Application` and `Device` for delivering firmware is extended to add build capabilities, but this is only available for firmware stored in container images at the moment:

```
spec:
    firmware:
        container:
            image: my-firmware:latest
            build:
                # An image reference to a container image used to build your project
                image: docker.io/myorg/firmware-builder-image:latest
                source:
                    git:
                        # Git repository URI
                        uri: https://github.com/myorg/example-project
                        # Project folder within repository
                        project: repo/sub/folder
                        # Git revision to use
                        rev: main
                # Arguments passed to the builder image
                args:
                - flag1
                - flag2
                artifact:
                    # Path to artifact generated by builder image
                    path: myartifact.bin
```

The builder image contains the tool chain required to build your project. Clearly, a public service offering Drogue Ajour could be exploited to run bitcoin mining, so to avoid that the Drogue Ajour installation can be configured with a set of applications that are allowed to build firmware.

NOTE: It doesn't have to be a Rust project! It can be any project capable of producing an binary artifact to be delivered to a device as long as your builder image can build it.

## Next steps

We've explored the options for providing an update service for the tiny edge. We've looked at the requirements for such as service, and announced the Drogue Ajour add-on for Drogue Cloud.

To learn more about Drogue Ajour, have a look at [the documentation](https://book.drogue.io/drogue-ajour/dev/index.html)
The [Drogue Cloud Sandbox](https://sandbox.drogue.cloud) is already running an instance of Drogue Ajour, and you can access the console [here](https://firmware.sandbox.drogue.cloud) using the same credentials as for the Drogue Cloud Sandbox.

There are still improvements to be made, and here are some of them

* Provide device examples for Zephyr doing firmware updates
* Support pushing build artifacts to Eclipse Hawkbit
* Retrieving build logs for builds in order to debug build failures
* ...

## Final notes

In this article series, we've gone through the entire chain from bootloader to firmware to gateway to the cloud, and shown you all the bits you can use to get firmware updates. In the below video you can see it all coming together:

* The bootloader from [part 1](https://blog.drogue.io/firmware-updates-part-1/)
* The updater application from [part 2](https://blog.drogue.io/firmware-updates-part-2/)
* The firmware update service from this post.

[![Firmware updates end to end](https://img.youtube.com/vi/l2pNJQOUbKs/0.jpg)](https://youtu.be/l2pNJQOUbKs)

If you have feedback or questions about any of the articles in this series, please reach out to us in the [forum](https://discourse.drogue.io/) or in the [chat](https://matrix.to/#/#drogue-iot:matrix.org).
