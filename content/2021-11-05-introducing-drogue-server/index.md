+++
title = "Introducing Drogue Cloud Server"
extra.author = "lulf"
+++

We love Kubernetes and the standardized API it brings to deploying applications in the cloud. For running a local instance of Drogue Cloud, [minikube](https://blog.drogue.io/minikube-roundtrip/) is already a great alternative. But sometimes, you just want to have an easy getting started experience, to reduce turnaround time when testing changes to Drogue Cloud itself, or for running in environments where Kubernetes is not available.

Read on to learn how you can run Drogue Cloud standalone.

<!-- more -->

# Appreciating Kubernetes

Drogue Cloud on Kubernetes automatically installs some dependencies of Drogue Cloud such as:

* [PostgreSQL](https://www.postgresql.org/)
* [Keycloak](https://www.keycloak.org/)
* [Kafka](https://kafka.apache.org/)

However, running outside of Kubernetes also means that you need to run or configure these services yourself. You can do this by installing the software manually and ensuring they are accessible locally. You can also rely on (tada!) containers using `docker-compose` or `podman-compose` if it is available on your platform, and we provide a [compose file](https://github.com/drogue-iot/drogue-cloud/blob/main/server/container-compose.yml) just for that purpose.


# Installing The Binary

One of the goals with running Drogue Cloud locally was to encapsulate the functionality in a single binary, `drogue-cloud-server`. You can [build it yourself](https://github.com/drogue-iot/drogue-cloud/tree/main/server#building), or you can [download prebuilt binaries]() for your platform.

# Running

Once installed, it's time to run. You can find the detailed instructions in [the book](https://book.drogue.io/drogue-cloud/dev/deployment/single-binary.html). It's important to note that for the time being, the Kafka, Keycloak and PostgreSQL instance must be reachable on localhost where the server is running.

Once you have the dependencies running, we can start the server:

```rust
./drogue-server run --enable-all
```

When starting, the server will perform the following steps (in order):

* Connect to PostgreSQL and run schema migration
* Connect to Keycloak and create an OIDC client
* Launch the services specified for the run command

Using `--enable-all` will run all Drogue Cloud services supported. At the time of writing, this is:

* REST API
* Device registry
* Device authentication
* User authentication
* HTTP endpoint
* MQTT endpoint

We plan to support all the Drogue Cloud services.

The server will print some info to the terminal on how to log into the server using the [drg](https://github.com/drogue-iot/drg) client, creating applications and devices, and publishing telemtry data:

```bash

```

## Other options

The server can also be run with other options, we'll quickly cover the most important:

* `--server-cert` and `--server-key` - enable TLS for the endpoints. Both arguments should refer to PEM-encoded files.
* `--bind-address` - bind to a different network interface (uses localhost by default).

# Connecting from Drogue Device

With the server running, testing it is super easy. If you have the hardware, there is already a lot of [examples](https://book.drogue.io/drogue-device/dev/examples.html#_drogue_cloud_connectivity_examples) that will work with the server out of the box (just point them to the correct IP of your server). 

If you don't have any supported microcontroller hardware, don't worry! We've got you covered with the [std cloud](https://github.com/drogue-iot/drogue-device/tree/main/examples/std/cloud) example that runs out of the box on any Linux/Mac OS X/Windows.

All you need to do is specify the device username and password in the example configuration (see the README for the example), edit the expected IP and port of your server instance, and run.

The output should look something like this when running the PC example:

```bash

```

# Summary

We have seen how you can get up and running with Drogue Cloud running on bare metal using a single binary. This enables quicker turneround times when developing Drogue Cloud, but also paves the way for running Drogue Cloud in more environments. Finally, we've seen examples of using [Drogue Device](https://github.com/drogue-iot/drogue-device) applications to connect.

But, this is only showing some of the potential. Future work includes:

* Support configuring PostgreSQL, Keycloak and Kafka host and credentials
* Support more Drogue Cloud services such as the CoAP endpoint, MQTT integration and Websocket integration services.


If you'd like to help out in these areas, join our [community](https://matrix.to/#/#drogue-iot:matrix.org)!
