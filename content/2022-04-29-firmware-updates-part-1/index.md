+++
title = "Firmware updates, part 1: Bootloader"
extra.author = "lulf"
+++

This is the first post in a series about doing device firmware updates (DFU) over the air (OTA) and continuous delivery of firmware for embedded devices. We'll explore the different parts of a complete end to end system with this capability.

This post will be about a fundamental component in such a system, the bootloader.

<!-- more -->

# Background

For a connected devices to be maintainable at scale, they must be able to update themselves just like any other software. However, managing a fleet of occasionally connected devices with little bandwidth require a different approach to delivering software updates to regular applications. One goal for Drogue IoT is to support the entire software update workflow on tiny devices, all the way from building your firmware to delivering it to the device, just like any other modern application.

# Bootloader

A fundamental component in an updatable system is the ability to boot different versions of an application. Making a generic bootloader is hard, because there is a big number of different possible device configurations. The complexity and size of a bootloader is also determined by it's functionality: a bootloader being able to retrieve firmware from a network has a bigger footprint than one that only loads an application from a fixed location in flash.

Since we're focused on IoT, we can assume that our devices have some form of network connectivity, but we do not wish to tie ourselves to any specific connectivity type. 

Moreover, for many applications, it's desirable to retrieve the updates while the application is running, which excludes some bootloader designs that do the firmware update within the bootloader.

What happens on power failure while updating? With devices installed in hard to reach locations, it is important that we can gracefully handle such a scenario and fall back to an existing version and try again. Likewise, should the new application not work properly, we want to allow falling back to the previous version known to work.

Finally, we want to be able to store firmware in external flash, which has potentially different page and transfer sizes than the internal on-chip flash.

# Introducing embassy-boot

The `embassy-boot` bootloader is a lightweight bootloader supporting firmware application upgrades in a power-fail-safe way, with trial boots and rollbacks. 

The bootloader consists of two parts, a platform independent part and a platform dependent part. The platform independent part is a standard Rust library (and there are unit tests using a in-memory 'flash' for testing correctness) and that can be used to build your custom bootloader. The platform-dependent part which ties the generic library to a specific hardware platform, such as nRF or STM32. This provides some hardware-specific functionality, for instance integration with nrf-softdevice or a watchdog timer for nRF devices.


<figure>
    <img src="dependencies.png" alt="Bootloader dependencies" />
    <figcaption>Bootloader Dependencies</figcaption>
</figure>


<figure>
    <img src="partitions.png" alt="Bootloader partitions" />
    <figcaption>Bootloader partitions</figcaption>
</figure>

## Initial boot

## Updating the firmware


## Swap algorithm

At the core of the bootloader is the bank swapping algorithm, which is not tied to any specific
platform. The algorithm keeps an internal state of the copy progress in flash, using 1 word per page to spread the writes.

Assume a flash size of 3 pages for the active partition, and 4 pages for the DFU partition.
The swap index contains the copy progress, as to allow detecting if copy is completed or not
on power failure. The index counter is represented within 1 or more pages (depending on total
flash size), where a page X is considered swapped if index at location (X + WRITE_SIZE)
contains a non-erased value. This ensures that index updates can be performed atomically and
avoid a situation where the wrong index value is set (page write size is "atomic").

The initial state of the Active, DFU and State partitions are shown below:

<figure>
    <img src="swap_initial.png" alt="Initial state" />
    <figcaption>Initial State</figcaption>
</figure>

The algorithm starts by copying 'backwards', and after the first step, the layout is
as follows:

<figure>
    <img src="swap_step1.png" alt="Copy state 1" />
    <figcaption>Copy State 1</figcaption>
</figure>

The next iteration performs the same steps

<figure>
    <img src="swap_step2.png" alt="Copy state 2" />
    <figcaption>Copy State 2</figcaption>
</figure>

And again until we're done

<figure>
    <img src="swap_final.png" alt="Final state" />
    <figcaption>Final State</figcaption>
</figure>

The reverting algorithm uses the swap index to check if images were swapped, or that
the application failed to mark the boot successful. In this case, the revert algorithm will
run.

The revert index is located separately from the swap index, to ensure that revert can continue
on power failure.

The revert algorithm works forwards, by starting copying into the 'unused' DFU page at the start.

Once the swap process is complete, the bootloader may jump to the application at beginning of the active partition.

This is a platform-specific step done in the `embassy-boot-stm32` or `embassy-boot-nrf` part of the bootloader.

NOTE: The new application is responsible for marking itself as successfully booted, otherwise the bootloader will attempt to revert to the previous application when restarted!

### Power fail safety

What happens if the swap process is interrupted during the copy? Can we still revert back to the old version in a half-copied state? Yes! After a power failure, the device may be in one of the following states during the update process:

* Firmware has been written, but not instructed to update. In this case, no action is taken and the old firmware will be used.
* Should swap, but swap is not complete. In this case, the bootloader will continue the swap operation from where it left off. The crucial step in this process is that the page copy progress state is written atomically.
* Swap is complete, but is still instructed to update. In this case, the bootloader will assume that something went wrong with the new application and will start to revert to the previous application.
* Should revert, but revert is not complete. Similar to the previous state, the revert process will continue where it left off until complete.

## Bootloader binary

## Application binary

## Alternatives

There are many existing bootloaders, like [mcuboot](https://www.mcuboot.com/), which probably has the best device and feature support. However, building and running a C based bootloader and adapting it to work with `embassy` is also not as nice as using Rust tooling and being able to reuse the hardware support already in embassy. 

Another bootloader with a similar approach to `embassy-boot` is [moonboot](https://jhbruhn.de/posts/moonboot/), which shares a similar design with a split responsibility between the bootloader and application but is even more generic (not tied to embassy, but also means more work to use) and (at the time of writing) not power fail safe. Clearly there is an opportunity of collaboration in the future.

# Summary
