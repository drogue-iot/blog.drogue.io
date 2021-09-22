+++
title = "Drogue Cloud: Release 0.7.0"
extra.author = "ctron"
description = "Summer is over, Drogue Cloud 0.7.0 is out: Kafka topic provisioning, CoAP support (GSoC), Websocket integration,  TimescaleDB, and much more!"
+++

The 0.7 release cycle ran through summer and with that, two things happened: people got on vacation, and
Google Summer of Code was going on. So instead of forcing out a release, we decided to just wait and release at a later
time. By the way, a decision we discussed during our open community calls: [come and join](https://calendar.google.com/calendar/u/0/embed?src=ofuctjec399jr6kara7n0uidqg@group.calendar.google.com)!

<!-- more -->

## Kafka topic provisioning

We wanted to split up the Kafka topics for quite a while now. This release, we finally did it! Each application has
its own event topic now. Yes, that means that command & control still only has a single topic, but we are getting there.

Using [Strimzi](https://strimzi.io/) under the hood, we already have an operator which can take care of creating Kafka topics, users, and
access roles. So we improved our device registry operator capabilities a bit, and created a (Strimzi) "topic operator",
which requests Kafka resources from Strimzi, tracks their state, and syncs that with our device registry.

The standard operator can also be replaced, to provision Kafka topics in a different way, should you need that. But
right now, Strimzi is the default implementation for this.

What do you get from that? Each time you create a new application, a Kafka topic will be provisioned, a user created, and
with that, you have direct access to the Kafka topic with your application. Of course, you can also still use the MQTT
integration. But wait, there is more …

## WebSocket integration

Getting data out of Drogue Cloud using MQTT is great. But sometimes WebSockets are just easier to use. And so we
created a [WebSockets implementation](https://book.drogue.io/drogue-cloud/dev/user-guide/integration-ws.html) that allows you to stream events. The payload format is of course using Cloud Events again.

We have a nice blog post, [explaining how to use the WebSocket integration](https://blog.drogue.io/websocket-integration/) too.

And while using a tool like `websocat` is trivial, `drg stream` can make it even simpler, taking care of endpoints and
authentication.

## CoAP endpoint

Implementing a [CoAP endpoint](https://book.drogue.io/drogue-cloud/dev/user-guide/endpoint-coap.html) was the Google Summer of Code project from Pranav. And although he had to learn lots of new
things, and faced some technical difficulties in addition to the overall complicated situation, he did an amazing job!
Including end-to-end testing! Well done!

If you are interested in reading more about his GSoC experience, I recommend reading [his GSoC report](https://pranav-bhatt.github.io/GSoC_2021/).

One thing that is missing, but that was not a requirement of the project, is the support for DTLS. This is an
important part but is planned to be added in the future. If you are interested, just let us or Pranav know.

## Trust anchor management

Another Google Summer of Code project was [adding device trust anchor management](https://vedangj044.github.io/blog/gsoc-community/) by Vedang, for `drg`, our command-line tool. This enables you to manage device trust
anchors, in the device registry, using a few simple commands.

We have a nice blog post on [using X.509 client certificates with drogue-cloud](https://blog.drogue.io/drg-trust/),
showing a few examples of the new functionality. And there is also a [summary post on GSoC phase 1](https://vedangj044.github.io/blog/gsoc-phase1/) from Vedang on how own blog.

## Improved deployment and updated dependencies

A lot of improvements have been made on deploying Drogue Cloud. The Helm Charts are more matured and provide more
knobs to tweak the deployment. Dependencies have been updated, which brings you Keycloak 15, Strimzi 0.25, and more.

As we now have dedicated Kafka topics, we could also improve the deployment of the standard example. It is still
deployed by default when using the script, but total optional and externally installable when using the Helm Charts
directly.

So you can just install the standard example application on your own cluster, connecting it to the Drogue Cloud
sandbox installation.

## Switch to TimescaleDB

Updating dependencies as a task for Drogue Cloud 0.7. And when we tried to upgrade InfluxDB, which we used as an
example so far, we ran into some real blockers. It turned out, it was easier to migrate from InfluxDB to TimescaleDB
than from InfluxDB 1 to 2. Also thanks to [Outflux](https://www.outfluxdata.com/), a great tool should you feel the same
pain.

With a few commands, we were able to migrate all our sandbox data to TimescaleDB. Recording it there from now on.
This also led to a new version of the pusher: the [drogue-postgres-pusher](https://github.com/drogue-iot/drogue-postgresql-pusher)
is a simple Knative image, which extracts data from Cloud Events, and pushes it to PostgreSQL or TimescaleDB.

## What's next

We got a nice and stable foundation now when it comes to IoT connectivity. There is always more you can add, and
we will. But for the next release of Drogue Cloud, we want to focus on polishing what we already have. This includes
support for metrics and tracing but also making it easier to integrate with existing solutions.

As Drogue Cloud is a platform to build your applications on, we will be creating more examples and tutorials to showcase
what is possible. A WiFi example in combination with Drogue Device is in the making. And we will also continue
working on the new [Eclipse IoT "Telemetry end-to-end" package](https://github.com/eclipse/packages/pull/288),
which makes use of Drogue Cloud too. But also shows how to integrate other, existing IoT projects
like [Eclipse Kura](https://www.eclipse.org/kura/) and [Eclipse Ditto](https://www.eclipse.org/ditto/).

As always, you are welcome to join and give the new version a try on [our sandbox](https://sandbox.drogue.cloud).

Oh, and we do have a [proper landing page](https://drogue.io/) now!

## Also see

* [Releases](https://github.com/drogue-iot/drogue-cloud/releases)
* [Public sandbox](https://sandbox.drogue.cloud)
* [Pranav's GSoC report](https://pranav-bhatt.github.io/GSoC_2021/)
* [Vedang's blog](https://vedangj044.github.io/blog/)
* [drg](https://github.com/drogue-iot/drg) (Drogue Cloud command line tool)
* [PostgreSQL/TimescaleDB pusher](https://github.com/drogue-iot/drogue-postgresql-pusher)
