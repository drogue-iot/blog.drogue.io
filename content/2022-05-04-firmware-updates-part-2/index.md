+++
title = "Firmware updates, part 2: Bootloader"
extra.author = "lulf"
+++

This is the first post in a series about doing device firmware updates (DFU) over the air (OTA) and continuous delivery of firmware for embedded devices. We'll explore the different parts of a complete end-to-end system with this capability.

This post will be about different transport mechanisms for doing firmware updates.

<!-- more -->

# Background

In the [previous post](https://blog.drogue.io/firmware-updates-part-1/), we explored the bootloader component required to do firmware updates in a power fail-safe way. Now that we've got a working bootloader, let's take a look at how we can get the firmware on the device itself.

TODO: The big picture

In this article we'll have a look at a few alternatives:

* WiFi
* Ethernet
* LoRaWAN
* LTE-M
* BLE
* USB
* Serial

Some of these transports work transparently with a IPv4/IPv6-based network, such as WiFi or Ethernet, and may therefore run directly on the embedded devices. We'll cover the gateway - cloud communication in the next blog post.

The other transport require some software on 'the other side' of the wire. For thesse, we'll demonstrate the `drgdfu` tool, which supports some of the above transports.

## The transport

The transport 
: USB, Serial and Bluetooth Low Energy (BLE).
