+++
title = "Introducing Drogue TLS"
extra.author = "lulf_and_bob"
+++

As we all know, the S in IoT stands for security. However, one reason IoT has a bad security track record is that it consumes a lot of resources and is not that easy to setup. One piece of this puzzle is having good client side libraries for doing TLS. Read on to learn more about how you can secure your network with Drogue IoT.

<!-- more -->

# What are the options?

There are no one solution fits all, and requirements for a hobby project is very likely different from installation in a secure facility. The approaches we've tried out so far is:

* No security - for hobby projects where you're just reporting temperature values to the cloud, security might not seem to be a big deal. However, if you're sending that data to some cloud service, chances are that the cloud service want your device to present some credentials. Since sending unencrypted credentials over a public network is a bad idea, this option is only viable in cases where you control the entire network where no one can sniff traffic. 
* Secured gateway to cloud - If you have a private network where you don't worry about someone sniffing out your credentials, you have the option of installing a local gateway or running a service on an existing gateway which forwards data over an encrypted link to a cloud service. This is useful if your device just cannot do encrypted traffic and you can be certain no one is sniffing credentials.
* Secure device to cloud - This requires your device to be capable of encrypting data, potentially supporting a standardized protocol like TLS (Transport Layer Security).

We'll explore the two latter options in this post.

## Secured gateway to cloud

If you have a too little RAM to do TLS on the device, and you are on a secure network, then you can setup a gateway between the device and cloud. One way to do that is to use [HAProxy](), which is fairly simple to setup. 

Here is an example configuration that you can append to the HAProxy config for proxying Device -HTTP-> HAProxy -HTTPS-> Drogue Cloud:

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
    balance     roundrobin
    http-request set-header Host http.sandbox.drogue.cloud
    server      static      http.sandbox.drogue.cloud:443 check sni str(http.sandbox.drogue.cloud) ssl verify none
```

The frontend configuration configures the incoming port bound to where devices can connect. The backend configuration 
configures the route to the Drogue IoT sandbox service. For the requests to be routed correctly, the HTTP Host header and the TLS Server Name (SNI) must be set to the destination host.

## Secured device to cloud

Doing TLS on devices is the preferred approach if you have the available RAM. Unfortunately, to use TLS in Rust embedded, is has been necessary to "pollute" your Rust project with an existing TLS library like mbed TLS, which works, but is not as nice; Until now. 

We've started a new implementation of TLS 1.3 in pure Rust and put it into the [`drogue-tls`]() crate. The goal is to implement the TLS 1.3 specification in pure Rust, using async. The project does not have any dependencies on other Drogue IoT projects, and can be used with any TCP stack, by implementing two traits used for I/O. 

### Where did all my RAM go?

We've tried to keep `drogue-tls` as small as possible. The overhead of the TLS connection is minimal, and highly correlated with the desired buffer size. To properly support any TLS compliant endpoint, the chosen buffer size should be at least the maximum TLS frame size, 16kB.

Since Drogue TLS is built using async, the futures require some stack allocation.

TODO: Introducing Drogue TLS

## Future work

Another piece of security we have not yet discussed is the security of the device itself. If the device is installed in a public location, you also need to make sure someone cannot simply connect to it physically and extract the credentials from flash. This is a topic on its own that we'll try to cover in a later post, but newer generation microcontrollers allow you to define secure areas of flash to prevent extraction of credentials.

# Summary

In this post we've gone through two options for securing device to cloud communication. One is by deploying a local gateway such as [HAProxy]() to handle the secure communication. The other is to encrypt data on the device, using the new [Drogue TLS]() Rust crate.
