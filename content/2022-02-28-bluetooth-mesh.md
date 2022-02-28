+++
title = "Bluetooth Mesh"
slug = "bluetooth-mesh"
extra.author = "bobmcwhirter"
+++

Bluetooth Mesh is a brokerless system for devices to communicate within
a local area. We've implemented a Bluetooth Mesh stack, in Rust, on top
of Drogue-Device, our async framework for embedded development.

<!-- more -->

# What's Bluetooth Mesh?

Bluetooth Mesh is a mesh topology using connectionless BLE advertising packets, to allow
a variety of devices to communicate with each other, using a low-power means, without
necessarily having knowledge about other members of the mesh.

The part that makes a _mesh_ is the fact it uses "managed flooding" to relay
packets possibly beyond the distance a single radio can transmit. Bluetooth,
as we know, has a somewhat limited range. Each member of the Mesh then, can
optionally extend the physical range of the entire mesh, by repeating or _relaying_
messages onward.

The "managed flooding" aspect means that every node capable of relaying does
attempt to relay any message it sees for the first time. Each message traversing
the mesh starts with a TTL (time-to-live) that is decremented each time a node
relays it onwards. In this way, hopefully every node sees every message at least
once, while also not repeatedly relaying messages it's seen more than once.

## Why Bluetooth Mesh and not Zigbee, Thread or something else?

We don't view the question of radio technology as an `XOR` condition. 
Instead, we are simply _starting_ with Bluetooth Mesh because we already
have familiarity with it, and the Bluetooth Consortium is pretty good about
making its specifications openly available.

We _did_ want a low-power method of connecting hundreds of battery-powered
devices, so WiFi is excluded from the potential candidates.

# The Moving Parts of a Mesh

## Devices, Elements & Models.

Every participant in the mesh is a _device_ (or _node_).  A device is a physical
thing, such as a _lightswitch_, an _mains power outlet_, or an _internet-connected chicken._

Every device has one or more _elements_, which represent an individually-addressable portion
of the device. A 2-gang outlet would have two _elements_: one element for the left socket, one
element for the right socket.  

Each element may have one or more _models_ associated with it.  A model in the Mesh
world is a predefined idea of a thing that can control or be controlled. There are models
defined for _on-off switches_ and _things that can be turned on and off_. 
Each model is identified by either a specification-defined UUID or a vendor-defined UUID.

Models allow data to be smaller over the air, and for all participants to know how to react
to messages, since they all share the same idea of what's being talked about. Bluetooth Mesh
messages are _not self-describing_. By using pre-defined models, a 4-byte message can be transmitted,
and all relevant devices know how to interpret those 4 bytes within the context of the
known model structure.

## Addresses

### Unicast addresses

Every element gets assigned a unique _unicast address_ within the mesh. Given this, a
single device with multiple elements ends up with multiple unicast addresses.  Using
the unicast address, messages can be targetting directly to an element on a device.
Any node that receives messages for another's address will still _relay_ those messages
onwards, helping them to find their destination.

### Virtual addresses

Bluetooth Mesh also defines a capability of having _virtual addresses_ which are not assigned
to any particular node. Instead, they act similar to IP mcast groups, which devices can send messages 
to, or monitor for messages being sent to it.

Virtual addresses are the part that allow devices to work in a publish/subscribe model without
any central broker.

## Keys

### Device Key

Every device has its own cryptographic key used during provisioning to create a device
key known by the provisioner (see below). For messages destined directly to the device for configuring
it by the provisioner, this device key is used to communicate securely.

### Network Keys

Every device brought into a mesh is given the network key used by the mesh. This allows multiple
distinct meshes to operate within radio earshot of each other, and not intermingle.

Devices on a mesh can also be added to a subnet of the mesh, with its own unique key, to further
restrict which devices can see which messages.

### Application Keys

A mesh may have one or more application keys. Each element/model on a device can use an appropriate
application key, to even further restrict which devices can see which messages.

Application keys provide a way to cryptographically keep your untrusted _internet-connected chicken device_ from
secretly unlocking your mesh-enabled _doorlock_. 

## Provisioner

A provisioner is a node that controls the membership and logical topology of the mesh. The
mesh does _not_ require a provisioner during normal operation. A provisioner is only required
to bring a new device into the mesh and to configure the topology through setting up the
various publish and subscribe rules for each device.

A stable mesh network can live happily without a provisioner, once established.

# Bluetooth Mesh on Linux is... challenging.

Your run of the mill Linux distribution probably includes `bluez`, which has out-of-the-box
support for Bluetooth Mesh. Sorta. Kinda.

Alas, the default `bluez` `bluetoothd` only supports Bluetooth Mesh over the GATT bearer,
which really isn't the ideal way to go about it. The GATT bearer uses connection-oriented
communications between the provisioner and the device being configured. It doesn't use the
normal underlying advertising-based bearer used by the devices to communicate with each other.

First you have to stop `bluetoothd` on your linux node, and instead install and run 
`bluetooth-meshd`, which can speak native advertising-bearer Bluetooth Mesh.  Instead of
using `meshctl`, you then use `mesh-cfgclient` to provision nodes.

# Let's walk through an example.

## Flash your device

Our example uses the current nRF52840-dk example that sets up a board with one active
button and one active LED.  Instructions for loading the code onto the board are 
[included in the repository](https://github.com/drogue-iot/drogue-device/tree/main/examples/nrf52/nrf52840-dk/ble-mesh).

## Discover unprovisioned devices

Once you have your linux stack working, you can listen for devices telling the world
they are unprovisioned, or not a part of any mesh network. As it learns from unprovisioned
devices, it'll display it on the console.

```shell
[mesh-cfgclient]# discover-unprovisioned on
Unprovisioned scan started
Scan result:
	rssi = -39
	UUID = 0EF817B94FA04859A4F7C80312CD724E
	OOB = A040
``` 

## Provision it

Once you have the UUID of the device wanting to join the mesh, you can provision it.

```shell
[mesh-cfgclient]# provision 0EF817B94FA04859A4F7C80312CD724E
Provisioning started
Assign addresses for 1 elements
Provisioning done:
Mesh node:
	UUID = 0EF817B94FA04859A4F7C80312CD724E
	primary = 00c4

	elements (1):
```

When this is completed, the current state of affairs is:

* The device has the network key
* The provisioner assigned it a _unicast address_
* The provisioner knows the device's key

## Inspect it

Further commands happen from the `config` menu, which you enter by typing `menu config`.

Next, type `target 00c4` (or the address assigned for your node), so that subsequent commands
are sent, using the device's key, to the device.

The `composition-get` command will retrieve the device's composition of elements and models.

```shell
[config: Target = 00c4]# composition-get
Received DeviceCompositionStatus (len 21)
Received composion:
	Feature support:
		relay: yes
		proxy: no
		friend: no
		lpn: no
	 Element 0:
		location: 0100
		SIG defined models:
		  Model ID	0000 "Configuration Server"
		  Model ID	1001 "Generic OnOff Client"
		  Model ID	1000 "Generic OnOff Server"
```

In this case, we see that this particular device does indeed support relaying. It also has
one element, which includes the `Configuration Server` model, which is the exact model that
can reply to commands such as `composition-get`.

It also includes the `Generic OnOff Client`, which in our case the `button1` on our board.
Additionally it includes the `Generic OnOff Server` which is connected to the LED on this board.

Naming within the Bluetooth Mesh specification can sometimes be ambiguously confusing.
A `client` tends to be a model that sends commands, while a `server` is a model that receives them.
Generally speaking. Sometimes. Mostly. Kinda.

**At this point, nothing happens on the board.**

## Distribute Application Key

Before we can make the button or LED do something useful, we need to create an application key.

We have to go `back` in `mesh-cfgclient` and create an application key for the primary network.
We have to provide an `index`, which gives everyone on the mesh a short name to refer to that
specific key. 

Indexes do not have to be sequential. In this case, we have app-keys in indexes 0, 1 and 2, but we
jump up and create an app-key in index 4.

```shell
[config: Target = 00c4]# back
[config: Target = 00c4]# appkey-create 0 4
[config: Target = 00c4]# keys
NetKey: 0 (0x000), phase: 0
	app_keys = 0 (0x000), 1 (0x001), 2 (0x002), 4 (0x004)
```

Currently, only the provisioner knows this key.  Now we must give it to the device we want
to have it using the `appkey-add` command.

```shell
[config: Target = 00c4]# appkey-add 4
No response for "AppKeyAdd" from 00c4
Received AppKeyStatus (len 4)
Node 00c4 AppKey status Success
NetKey	0 (0x000)
AppKey	4 (0x004)
Received AppKeyStatus (len 4)
```

You may notice that `mesh-cfgclient` first reported "No response" but the we get a response.
That's life on a mesh for you.  Since Bluetooth Mesh is connectionless, a lot of timeouts
are involved. `mesh-cfgclient` decided the command had timed out, but then device sent
a response a few milliseconds later and everything is _fine_.

At this point, the device has application key #4, but has no idea what to do with it.

## Bind Application Key

Application keys can be _bound_ to a model within an element. This tells the device that
for messages to or from that model within the element, it should use a specific key for
crypto.

Here we bind it to both the _client_ and the _server_ on the board.

```shell
[config: Target = 00c4]# bind 00c4 4 1001
Received ModelAppStatus (len 7)
Node 00c4: Model App status Success
Element Addr	00c4
Model ID	1001 "Generic OnOff Client"
AppIdx		4 (0x004)

[config: Target = 00c4]# bind 00c4 4 1000
Received ModelAppStatus (len 7)
Node 00c4: Model App status Success
Element Addr	00c4
Model ID	1000 "Generic OnOff Server"
AppIdx		4 (0x004)
 [config: Target = 00c4]#
```

## Pub and Sub

Thus far, the client and the server know how to encrypt and decrypt messages related
to their models using application key #4. But how do they know where to send messages or receive them?

They don't. Yet.

### Virtual Address

Since we don't want to bind this particular button to that particular LED, we want to use a
_virtual address_.  This will allow us to have tons of buttons turning tons of LEDs on and off, if we
wanted. 

Imagine a 3-way lightswitch. You may a switch at the top of your stairs, and another at the bottom.
Either switch can turn the several lights going up the stairwell on or off.  Here, a virtual address
might represent "control of the stairway lighting".

Within `mesh-cfgclient` we use `virt-add` which synthesizes a new virtual address for us and registers
it within the provsioner.

```shell
[config: Target = 00c4]# virt-add
	Virtual addr: bda5, label: 1d3ffe292725f6cb64eaed53dba1d979
```

Still, nobody knows about it, beside the provisioner.

### Publishing

Now, we tell the button on the board to _publish_ its messages to that virtual address:

```shell
[config: Target = 00c4]# pub-set 00c4 bda5 4 1 1 1001
No response for "ModelPubVirtualSet" from 00c4
Node 00c4 Publication status Success
Model ID	1001 "Generic OnOff Client"
	Element: 00c4
	Pub Addr: bda5
	Model: 1001
	App Key Idx: 4 (0x004)
	TTL: ff
Period		100 ms
Rexmit count	1
Rexmit steps	0
```

Now, when we push or release the button, it'll be shooting messages onto the Mesh addressed to
`bda5`.

But if a message falls in a forest and nobody is around to hear it, does it turn a light on?

No.

### Subscribing

Next, we have to tell our LED to *listen* for messages sent to the same virtual address.

```shell
[config: Target = 00c4]# sub-add 00c4 bda5 1000
Received ModelSubStatus (len 7)

Node 00c4 Subscription status Success
Element Addr	00c4
Model ID	1000 "Generic OnOff Server"
Subscr Addr	bda5
```

# Success!

Here, I've done the above process with two boards, so that we can see how a virtual address
allows communication between several devices that have no knowledge of each other.

{{ vimeo(id="683002256", class="vimeo") }}

# Firmware

For the most part, the firmware only has to adapt Drogue-Device actor messages to/from 
domain-specific types, such as `LEDMessage::On` or `ButtonMessage::Pressed` to 
Bluetooth Mesh model messages, such as `GenericOnOffMessage::Set(...)`.

The full example code is in [our repository.](https://github.com/drogue-iot/drogue-device/blob/main/examples/nrf52/nrf52840-dk/ble-mesh/src/main.rs)

# That seems complex...

Yes, we all agree the various commands and workflow to accomplish this through
`mesh-cfgclient` is complex. Ultimately `mesh-cfgclient` is not a super-friendly
application meant for end-users. Instead, it's the barest plumbing possible to
accomplish all the steps required to provision and configure devices.

We assume in a real-world scenario, someone would create enough porcelain
around it all to allow an end-user to think more semantically about lightswitches,
stairways and internet-connected chickens.

