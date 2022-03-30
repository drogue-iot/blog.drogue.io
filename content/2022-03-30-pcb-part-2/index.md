+++
title = "The making of a PCB: Part 2"
extra.author = "ulf"
+++

Producing 1 PCB is very different from producing 500 PCBs, especially considering the logistics of assembling and testing. In this blog post, we'll continue the story from [last time]().

<!-- more -->

# The final version

The initial revision of the PCB worked for the most part. All the critical sensors appeared to be working, but we found a few things to improve on the design, and a design review with [James Munns]() helped identifying some additional improvements.

## Button placement

The button placement was too close to the thumb when operating with one hand. The fix for this was to place them closer to the center.

## Battery measurement

The battery measurement circuit was wrong, and caused inputs outside the ADC range of the microcontroller. Fixing this involved using different resistor values for the circuit.

## Accelerometer orientation

In order to make the sensor output compatible with both the Adafruit Feather nRF52840 Sense and Feather Adafruit nRF52840 Express, we switched the orientation of the accelerometer to make sure the output axes values was the same.

## Board outline

The sharp edges are a bit annoying when using the device. Given that this device is supposed to be used in a game like setting, we added some rounded corners.

In addition,we added a few holes so that we could add a string and users would be able to wear the device around their neck.

To avoid any potential for radio interference of the Adafruit from the PCB, we also added a keepout zone beneath the radio for good measure.

## Power

In the initial revision, the power switch was connected to the EN pin of the Adafruit, as this would allow us to use the switch for LiPoly batteries as well. However, even in this mode the device draws power, so we decided to have the switch control the battery power source directly and remove the jumper pins we placed there to prevent accidental reverse current.

Instead, we added a Schottky diode to prevent the case of reverse current with both batteries and USB power connected.

## Pairing with BLE Mesh

Since the devices are going to be part of a BLE Mesh network and paired with a single application, we need a way to provision them and identify them. In order to do that, we decided to make room for a QR code sticker in the moddle of the board. Then, upon flashing the device, we would set the expected mesh node UUID in the device firmware, and create a mapping between the UUID and a QR code. 

The pairing process will then use this data to ensure sensor data is routed to the expected application.

## Wiring and traces

In the initial revision, the outline made no use of ground vias, a technique that adds vias to the board in order to reduce the path from components to ground.


# Next revision

Having fixed the defects, we also intially removed the accelerometer footprint from the PCB, and revision 3.0 did not include it. However, to avoid risking supply chain issues, we quickly decided to add it back for a final revision 3.5.

# Test jig

# PCB assembly

For someone not familiar with PCB manufacturing, getting to the point where we could order fully assembled devices was a challenge. Most manufacturers have some online quotation service, but ultmiately it was easier talking to their sales departement and get the necessary help. We decided to order 5 fully assembled PCBs from RushPCB, who were very helpful and guiding us to retrieve the information they needed.

In particular, assembly drawings and instructions are things you don't need to think about when assembling the PCB yourself.

# Summary

The final revision 
