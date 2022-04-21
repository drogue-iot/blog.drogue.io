+++
title = "Firmware updates, part 1: Bootloader"
extra.author = "lulf"
+++

This is the first post in a series about doing device firmware updates (DFU) over the air (OTA) and continuous delivery of firmware for embedded devices. We'll explore the different aspects of a system with this capability.

This post will be about a fundamental component in such a system, the bootloader.

<!-- more -->

# Background

For a connected device to be maintained at scale, it must be able to update itself just like any other software. However, managing a fleet of occasionally connected devices with little bandwidth require a different approach to delivering software updates to regular applications. One goal for Drogue IoT is to support the entire software update workflow on tiny devices, all the way from building your firmware to delivering it to the device.

# Bootloader

A fundamental component in an updatable system is the ability to boot different versions of an application. There are many existing approaches to this, like [mcuboot](). Making a generic bootloader is hard, because there is a huge number of different possible device configurations. 




