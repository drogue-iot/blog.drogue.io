+++
title = "Expanding the set of LoRaWAN drivers"
extra.author = "lulf"
+++

Earlier we've seen examples of drogue device using LoRaWAN using an STM32 discovery board. Since
then, the hardware support in the community has improved even more.

<!-- more -->

There are multiple efforts within Rust Embedded to improve LoRaWAN support. This post will walk
through the state of the ecosystem and introduce some of the crates being worked on, and Drogue
Device's approach.

# Hardware support

The LoRaWAN capable hardware is dominated by Semtech-based radios. Up until now, the Sx127x family
of radios have been the most common, with an SPI interface for accessing the radio state. This radio
is well supported in Rust, and there are several crates for accessing it, such as [sx127x_lora]()
and [radio-sx127x](). Drogue device uses a variant of the first.

Another variant is the Sx126x family of radios, which is a more power efficient and has a smaller
footprint. This is also used in the STM32WL family of chips, using a special SPI peripheral for
accessing it. Recently, there have been efforts to add support for these chips and radios both in the
[stm32wl-hal]() crate, and with a corresponding async version in [embassy](), where the radio peripheral 
is reusing much of the same code. The STM32WL chip is also used in the new [Generic Node]() sensor
from [The Things Industries (TTI)](). As drogue device is based on embassy, the radio peripheral can
be used with this chip there as well.

# The link layer

The [rust-lorawan]() crate implements a LoRaWAN compliant MAC layer, and allows implementing a
LoRaWAN driver using any radio peripheral, and this is in fact already part of drogue-device for the
sx127x based radios.

With the new stm32wl-hal, the [lorawan-wl]() crate was created, and integrates the STM32WL radio with rust-lorawan. Likewise, drogue device integrates the same peripheral in embassy with the rust-lorawan crate.

# Embassy



# Drogue device

# Summary

