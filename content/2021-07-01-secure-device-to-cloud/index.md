+++
title = "Introducing Drogue TLS"
extra.author = "bob_and_ulf"
+++

As we all know, the S in IoT stands for security. However, one reason IoT has a bad security track record is that it consumes a lot of resources and is not that easy to setup. One piece of this puzzle is having good client side libraries for doing TLS. Read on to learn more about how you can secure your device to cloud communication.

<!-- more -->

# What are the options?

There are no one solution fits all, and requirements for a hobby project is very likely different from installation in a secure facility. The approaches we've tried out so far is:

* No security - for hobby projects where you're just reporting temperature values to the cloud, security might not seem to be a big deal. However, if you're sending that data to some cloud service, chances are that the cloud service want your device to present some credentials. Since sending unencrypted credentials over a public network is a bad idea, this option is only viable in cases where you control the entire network where no one can sniff traffic. 
* Secured gateway to cloud - If you have a private network where you don't worry about someone sniffing out your credentials, you have the option of installing a local gateway or running a service on an existing gateway which forwards data over an encrypted link to a cloud service. This is useful if your device just cannot do encrypted traffic and you can be certain no one is sniffing credentials.
* Secure device to cloud - This requires your device to be capable of encrypting data, potentially supporting a standardized protocol like TLS (Transport Layer Security).

We'll explore the two latter options in this post.

## Secured gateway to cloud

If you have a too little RAM to do TLS on the device, and you are on a secure network, then you can setup a gateway between the device and cloud. One way to do that is to use [HAProxy](http://www.haproxy.org/), which is fairly simple to setup. 

Here is an example configuration that you can append to the HAProxy config for proxying `Device -http-> HAProxy -https -> Drogue Cloud`:

```
#---------------------------------------------------------------------
# main frontend which devices connect to
#---------------------------------------------------------------------
frontend main
    bind *:80
    default_backend drogue

#---------------------------------------------------------------------
# Drogue IoT backend
#---------------------------------------------------------------------
backend drogue
    balance      roundrobin
    http-request set-header  Host http.sandbox.drogue.cloud
    server       static      http.sandbox.drogue.cloud:443 check sni str(http.sandbox.drogue.cloud) ssl
```

The frontend configuration configures the incoming port bound to where devices can connect. The backend configuration 
configures the route to the Drogue IoT sandbox service. For the requests to be routed correctly, the HTTP Host header and the TLS Server Name (SNI) must be set to the destination host.

## Secured device to cloud

Doing TLS on devices is the preferred approach if you have the available RAM. Unfortunately, to use TLS in Rust embedded, is has been necessary to "pollute" your Rust project with an existing TLS library like mbed TLS, which is the old approach taken by [drogue-tls](https://github.com/drogue-iot/drogue-tls).

We've started a new implementation of TLS 1.3 in pure Rust and replaced the mbedTLS-based version it. The goal is to implement the TLS 1.3 specification in pure Rust, with async support. The project does not have any dependencies on other Drogue IoT projects, and can be used with any TCP stack, by implementing two traits used for I/O. In fact, there are implementations of these traits for some common runtimes, and you can find examples of using Drogue TLS with [tokio](https://github.com/drogue-iot/drogue-tls/tree/main/examples/tokio) and [embassy + smoltcp](https://github.com/drogue-iot/drogue-tls/tree/main/examples/embassy), with trait implementations for [futures](https://github.com/drogue-iot/drogue-tls/tree/main/src/lib.rs) I/O traits as well.

The project is still under development, and critical features such as certificate validation and client certificate authentication is still on the [TODO list](https://github.com/drogue-iot/drogue-tls/issues). 

### Why TLS 1.3?

TLS 1.3 is simpler and easier to implement than TLS 1.2. This is partially because there are only a single cipher suite that needs implementing (TLS_AES_128_GCM_SHA256), which is also not too resource demanding for embedded devices, but also because there are fewer mandatory extensions.

### Where does my RAM go?

We've tried to keep the memory footprint of `drogue-tls` small. The overhead of a TLS connection is minimal, and correlates with the desired buffer size. To properly support any TLS compliant endpoint, the chosen buffer size should be at least the maximum TLS frame size, 16kB. However, if you know your application will not be sending much data, you can get away with less. Given the focus on embedded, the primary goal has been to make it small rather than fast.

Since Drogue TLS is built using async, the futures also require some stack allocation. However, since [embassy](https://github.com/embassy-rs/embassy) uses static futures for the tasks, we know the memory usage at compile time (BSS).

As an example, memory usage of [embassy + smoltcp](https://github.com/drogue-iot/drogue-tls/tree/main/examples/embassy) binary, compile to the `x86_64-unknown-linux-gnu` target, looks like this without TLS:

```
$ size target/release/ping-embassy-net
   text    data     bss     dec     hex filename
1094295   42040   16880 1153215  1198bf target/release/ping-embassy-net
```

Whereas with TLS enabled using a 16 kB record buffer, the memory usage increases:

```
$ size target/release/ping-embassy-net
   text    data     bss     dec     hex filename
1175083    44392   37296 1256771  132d43 target/release/ping-embassy-net
```

As you can see, the BSS overhead is about 20 kB when enabling TLS with a 16 kB record buffer. The same overhead can be observed when compiling to ARM architectures.

We have not yet look at flash usage, so there are probably improvements to be made there as well.

### What about Drogue Device?

Drogue Device now supports TLS, and the example using [micro:bit + ESP8266 WiFi](https://github.com/drogue-iot/drogue-device/tree/main/examples/nrf52/microbit-esp8266) is now using Drogue TLS to connect directly to the [sandbox](https://sandbox.drogue.cloud/).

We want Drogue Device to provide a simple way to get started with IoT, so having a simple API to use networking is important. As an example, the following code joins the wifi network, creates a regular `Socket` before wrapping it in a `TlsSocket` which implements the same trait as a regular `Socket`.

```rust
let mut wifi = device.wifi.mount((), spawner);
wifi.join(Join::Wpa {
    ssid: WIFI_SSID,
    password: WIFI_PSK,
})
.await
.expect("Error joining wifi");
log::info!("WiFi network joined");

let socket = Socket::new(wifi, wifi.open().await);
let socket = TlsSocket::wrap(socket,
    TlsContext::new()
        .with_rng(rng)
        .with_server_name(HOST));
```

Because they implement the same trait, it's easy to write logic that does not have to care about the connection being over TCP or TLS.

## Future work

As mentioned, Drogue TLS is still under development, with some critical features missing. However, we hope to see interest from the community and hope to collaborate on improving the TLS support for Rust embedded.

Another piece of security we have not yet discussed is the security of the device itself. If the device is installed in a public location, you also need to make sure someone cannot simply connect to it physically and extract the credentials from flash. This is a topic on its own that we'll try to cover in a later post, but newer generation microcontrollers allow you to define secure areas of flash to prevent extraction of credentials.

# Summary

In this post we've gone through two options for securing device to cloud communication. One is by deploying a local gateway such as [HAProxy](http://www.haproxy.org/) to handle the secure communication. The other is to encrypt data on the device, using the new [Drogue TLS](https://github.com/drogue-iot/drogue-tls) library. Finally, we've seen how you can use Drogue TLS with [Drogue Device](https://github.com/drogue-iot/drogue-device).
