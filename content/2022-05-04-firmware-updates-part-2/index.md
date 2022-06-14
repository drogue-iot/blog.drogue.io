+++
title = "Firmware updates, part 2: Protocols"
extra.author = "lulf"
+++

This is the first post in a series about doing device firmware updates (DFU) over the air (OTA) and continuous delivery of firmware for embedded devices. We'll explore the different parts of a complete end-to-end system with this capability.

This post will be about the different protocols and how to do firmware updates for them.

<!-- more -->

# Background

In the [previous post](https://blog.drogue.io/firmware-updates-part-1/), we explored the bootloader component required to do firmware updates in a power fail-safe way. Now that we've got a working bootloader, let's take a look at how we can get the firmware on the device itself.

TODO: The big picture

In this article we'll have a look at the most familiar connectivity options:

* WiFi/Ethernet/LTE-M
* LoRaWAN
* BLE (Bluetooth Low Energy)
* Serial/UART

# Protocols/transports

Some of the protocols can easily be adapted to a IPv4/IPv6-based network, and may therefore run directly on the embedded devices. Other protocols like BLE require custom software or services to translate between the protocol and the IP network. For BLE and serial, we'll demonstrate the `drgdfu` tool. For LoRaWAN, integration with the network provider such as [TTN](https://www.thethingsnetwork.org/) handles the translation.

## IP-based

If your devices has WiFi and/or Ethernet capabilities, then talking to the cloud becomes a lot easier. The downsides are power usage and range (WiFi) or the need for using wires (Ethernet). Even with these protocols, you need a TCP/IP implementation. Many WiFi adapters already provide a TCP/IP implementation which you interact with using [AT commands](https://www.espressif.com/sites/default/files/documentation/4b-esp8266_at_command_examples_en.pdf). [Drogue Device](https://github.com/drogue-iot/drogue-device) contains drivers for the [ESP8266](https://en.wikipedia.org/wiki/ESP8266) and [eS-WiFi](https://www.inventeksys.com/es-wifi-support/) that you can use with [embassy](https://embassy.dev/). 

In the cases where you don't have a TCP/IP implementation, you can use an open source implementation. In the C world, there is [LwIP](https://savannah.nongnu.org/projects/lwip/), but in the world _we_ care about (Rust), there is [smoltcp](https://github.com/smoltcp-rs/smoltcp). 

Rather than going via an additional process for retrieving the firmware data, we can connect directly to the cloud service. In this case, the [embedded-update](https://crates.io/crates/embedded-update) defines a firmware update protocol than can be plugged in to different firmware update services and devices. It performs the firmware update protocol process if you supply it with implementations for the `UpdateService` and `FirmwareDevice` traits. At the time of writing, `drogue-device` contains an implementation of `UpdateService` for `Drogue Cloud`, and an implementation of `FirmwareDevice` based on `embassy-boot`.

For the full details, have a look at the [STM32L4 (WiFi)](https://github.com/drogue-iot/drogue-device/tree/main/examples/stm32l4/iot01a/wifi) example.

We'll have a closer look at the `Drogue Cloud` firmware update service in the next article in this series.

## LoRaWAN

LoRaWAN is a great protocol for devices that infrequently sends sensor data from devices. Devices can also use little power and transmit data over long distances compared to WiFi or BLE. However, LoRaWAN networks also have bandwidth limitations. For instance, doing a 64kB firmware update using the [TTN free tier](https://www.thethingsnetwork.org/) network would take 4 days! Things look a bit better if running your own network, but you'll still be limited by regulated bandwith usage and the lower limit is 4 hours for 64kB.

The being said, spending 4 hours or 4 days to update firmware of IoT sensors might not be problem for many applications, and as long as the firmware can be fetched at such a low pace, it works as expected.

Similar to the IP based example, we can use the same `embedded-update` crate to enable firmware updates for our lorawan device.

## LTE-M / NB-IoT

LTE-M/NB-IoT device consume more power than LoRaWAN devices. On the other hand, the network coverage and bandwidth is a lot higher. With LTE-M you can get up to 1 mbit downlink speeds (under ideal conditions!), which allows you to quickly download firmware when there is an update. These devices often contains a TCP/IP implementation allowing you to connect directly to firmware update services. If you can live with the higher power usage and the additional cost (as it usually involves a montly subscription to the provider), this transport can be a good way to ensure you have firmware update capability for your device.

We don't have any examples for LTE-M / NB-IoT at present, but in general the approach would be similar to IP based networks.

## BLE / Thread / ZigBee

These standards are designed to work well with low power devices, but with limited range and not directly interoperable with an IP network (6LoWPAN allows that, but it is IPv6 only and not widely supported). Because use of that, they need a gateway component to reach the internet. Since Thread and ZigBee is not well adopted in the Rust embedded community, we'll consider BLE which is also the more complex to translate.

For BLE, there is no standard firmware update service UUID defined, so we have defined a custom service of our own, implemented using the `nrf-softdevice` crate supported by the nRF52 chip families:

```rust
// The FirmwareUpdate GATT service
#[nrf_softdevice::gatt_service(uuid = "00001000-b0cd-11ec-871f-d45ddf138840")]
pub struct FirmwareService {
    /// Version of current running firmware
    #[characteristic(uuid = "00001001-b0cd-11ec-871f-d45ddf138840", read)]
    version: Vec<u8, 16>,

    /// Max firmware block size for device
    #[characteristic(uuid = "00001002-b0cd-11ec-871f-d45ddf138840", read)]
    mtu: u8,

    /// State control
    #[characteristic(uuid = "00001003-b0cd-11ec-871f-d45ddf138840", write)]
    control: u8,

    /// Version being written
    #[characteristic(uuid = "00001004-b0cd-11ec-871f-d45ddf138840", write, read)]
    next_version: Vec<u8, 16>,

    /// Current write offset
    #[characteristic(uuid = "00001005-b0cd-11ec-871f-d45ddf138840", read)]
    offset: u32,

    /// Firmware data to be written
    #[characteristic(uuid = "00001006-b0cd-11ec-871f-d45ddf138840", write)]
    firmware: Vec<u8, 64>,
}
```

The `drgdfu` tool implements a `GATT` client for interacting with this service, whereas `drogue-device` provides an implementation that adapts the above service to the `FirmwareDevice` trait from `embedded-update`.

## Serial

The good old serial interface is a simple but flexible way to upload firmware. The downside of serial is of course that it is wired, but it can be a nice 'fallback' for cases where you need to go on site anyway as it's quite fast and efficient.

Besides, the `embedded-update` crate provides a `SerialUpdateService` implementation that works with traits from `embedded-io` and using `postcard` with a fixed-frame format. This means that the protocol can be used not only for serial, but also USB, TCP and UDP if desired. 

## An example

Let's look at an example running firmeware updates over a BLE connection. The full example can be found [here](https://github.com/drogue-iot/drogue-device/tree/main/examples/nrf52/microbit/ble).

You can find more examples in the same git repository:

* [TCP/IP + STM32 L4 IoT01a (WiFi) using Drogue Cloud]()
* [TCP/IP + STM32 H7 Nucleo-144 (Ethernet) using Drogue Cloud]()
* [LoRaWAN + LoRA-E5 Mini using The Things Network + Drogue Cloud]()

Common to all is that they use the `embassy-boot` bootloader for the ability to swap firmware images.

# Next steps
