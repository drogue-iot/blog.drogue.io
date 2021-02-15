+++
title = "From drogue-device to drogue-cloud"
extra.author = "ulf_and_jens"
+++

In previous posts we've seen how to [run drogue-cloud](), how to [use LoRaWAN in rust](), and the introduction of the [drogue-device](). In this post, we'll tie this together and walk through the process of running LoRa on drogue-device, sending data to drogue-cloud. 

<!-- more -->

# Recap

* About drogue-cloud
* About LoRa
* About drogue-device

# Device drivers on drogue-device

* Actor, interrupts and packages
* Rak811 device driver overview

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
