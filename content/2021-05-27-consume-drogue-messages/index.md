+++
title = "Consuming Drogue messages with a Knative Service"
description = "How to extend the drogue platform with knative functions"
extra.author = "jcrossley"
+++

For most IoT developers, drogue-cloud is designed to be used "as a
service". But if you have admin access to the cluster on which
drogue-cloud is running, i.e. you installed it yourself on [minikube]
or [kind], it's possible to "extend" the platform by triggering a
function each time an event from a device is processed.

<!-- more -->

This article assumes you've [installed drogue-cloud per the
instructions](https://book.drogue.io/drogue-cloud/dev/deployment/index.html),
along with a few of its pre-requisites. Specifically, we'll be using
[kubectl] and [func], discussed below, to deploy our event source and
service. And we'll use [drg] and [HTTPie] to test it.


# The KafkaSource

We need two things: a knative `KafkaSource` to forward the Drogue
[CloudEvents], and a knative `Service` to receive them. Because
drogue-cloud is built atop kafka and knative-eventing, the
`KafkaSource` custom resource definition (CRD) is already on your
cluster, along with the topic and bootstrap server listed in the
following YAML. The only thing you may want to change is the
`consumerGroup`, to ensure the "sink" receives all messages. If you
create more than one `KafkaSource` for the same topic, you probably
want to give each a unique `consumerGroup` name.

~~~yaml
apiVersion: sources.knative.dev/v1alpha1
kind: KafkaSource
metadata:
  name: drogue-messages
spec:
  consumerGroup: some-unique-name
  bootstrapServers:
    - kafka-eventing-kafka-bootstrap.knative-eventing.svc:9092
  topics:
    - knative-messaging-kafka.drogue-iot.iot-channel
  sink:
    ref:
      apiVersion: serving.knative.dev/v1
      kind: Service
      name: drogue-addict
~~~

Assuming you put that YAML in a file named `source.yaml` you would
apply it like so:

```shell
kubectl apply -f source.yaml
```

You can then observe that its status won't become "Ready" until we
create the `drogue-addict` service, configured as its "sink" above:

```shell
kubectl get kafkasource
```


# Our Knative Service, built by func

The [func] project aims to simplify the creation of Knative Services
that respond to [CloudEvents]. It was designed to be run as either a
self-contained CLI or a [`kn` plugin]. Unpack the [appropriate
binary](https://github.com/boson-project/func/releases/latest)
somewhere on your `$PATH` and run the following to get started:

```shell
func create drogue-addict && cd drogue-addict
```

Though it supports an ever-expanding [variety of popular
languages](https://github.com/boson-project/func/blob/main/docs/guides/developers_guide.md),
its default is Node.js, which will serve our purposes for now, since
we're just going to log the event we receive, along with the request
context, just to show what data we have to work with.  Overwrite the
entire `index.js` file it generated with the following:

~~~javascript
/**
 * Log the drogue event 
 *
 * @param {Context} context the invocation context
 * @param {Object} event the CloudEvent 
 */
function handle(context, event) {
  context.log.info("context");
  console.log(JSON.stringify(context, null, 2));
  context.log.info("event");
  console.log(JSON.stringify(event, null, 2));
};

module.exports = handle;
~~~

We need to build our function image. Unfortunately, [func] currently
requires a docker daemon for this. If you're using [podman], you can
simulate that like so:

```shell
podman system service --time=0 tcp:localhost:1234 &
export DOCKER_HOST=tcp://127.0.0.1:1234
```

With the daemon configured, we can now deploy our function. We'll need a
docker registry to store our image:

```shell
func deploy --registry docker.io/<YOUR_ACCOUNT>
```


# Testing our function

With any luck, when we publish an event from our "device" to the
drogue-cloud, that event will be logged by our Node.js function. We can
simulate a device sending data using a handy script in the same
directory from which you initially installed drogue-cloud.

The script will invoke the Drogue CLI, `drg`, to first authenticate
you, and then create identifiers for your app and device, before
finally publishing some fake data for that device.

The first time you run it, you should be redirected to your browser
and prompted for a username and password. Use `admin` and
`admin123456`, respectively.

```shell
cd <DROGUE_INSTALL_DIR>
./scripts/publish.sh
```

Assuming you see `HTTP/1.1 202 Accepted` output from the script, you
should see a pod fire up containing our function:

```shell
kubectl get pod
```

You should actually see _two_ pods, one for our `drogue-addict`
service, and one for the `KafkaSource` since its sink finally showed
up. But because knative services will scale down to 0 when idle (to
save you money!), the pod running our function will only hang around
for a few minutes. While it's up, you can run this to view the
messages logged by our function:

```shell
kubectl logs -l serving.knative.dev/service=drogue-addict -c user-container --tail=-1
```

Fingers crossed, you'll see something resembling this:

```shell
{"level":30,"time":1621899419651,"pid":20,"hostname":"drogue-addict-00001-deployment-799ddc69c7-dpqgm","reqId":"req-2","req":{"method":"POST","url":"/","hostname":"drogue-addict.default.svc.cluster.local","remoteAddress":"127.0.0.1","remotePort":35672},"msg":"incoming request"}
{"level":30,"time":1621899419655,"pid":20,"hostname":"drogue-addict-00001-deployment-799ddc69c7-dpqgm","reqId":"req-2","msg":"context"}
{
  "query": {},
  "body": {
    "temp": 42
  },
  "headers": {
    "host": "drogue-addict.default.svc.cluster.local",
    "user-agent": "Go-http-client/1.1",
    "content-length": "12",
    "accept-encoding": "gzip",
    "ce-application": "app_id",
    "ce-device": "device_id",
    "ce-id": "937d586a-e5dc-441d-96e8-24a74e561965",
    "ce-instance": "drogue",
    "ce-partitionkey": "app%5Fid/device%5Fid",
    "ce-source": "drogue://app%5Fid/device%5Fid",
    "ce-specversion": "1.0",
    "ce-subject": "anything",
    "ce-time": "2021-05-24T23:36:57.448964410+00:00",
    "ce-traceparent": "00-f81b71b968f77384e065fcf3aa33e65d-120c506cd2ce781e-00",
    "ce-type": "io.drogue.event.v1",
    "content-type": "application/json",
    "forwarded": "for=172.17.0.9;proto=http",
    "k-proxy-request": "activator",
    "traceparent": "00-f81b71b968f77384e065fcf3aa33e65d-6bbbbe6e14ee9dd8-00",
    "x-forwarded-for": "172.17.0.9, 172.17.0.2",
    "x-forwarded-proto": "http",
    "x-request-id": "3883fd1f-cfa6-4297-90be-f2a78e34d32c"
  },
  "method": "POST",
  "httpVersion": "1.1",
  "httpVersionMajor": 1,
  "httpVersionMinor": 1,
  "log": {},
  "cloudevent": {
    "id": "937d586a-e5dc-441d-96e8-24a74e561965",
    "time": "2021-05-24T23:36:57.448Z",
    "type": "io.drogue.event.v1",
    "source": "drogue://app%5Fid/device%5Fid",
    "specversion": "1.0",
    "datacontenttype": "application/json",
    "subject": "anything",
    "application": "app_id",
    "device": "device_id",
    "instance": "drogue",
    "partitionkey": "app%5Fid/device%5Fid",
    "traceparent": "00-f81b71b968f77384e065fcf3aa33e65d-120c506cd2ce781e-00",
    "data": {
      "temp": 42
    }
  }
}
{"level":30,"time":1621899419656,"pid":20,"hostname":"drogue-addict-00001-deployment-799ddc69c7-dpqgm","reqId":"req-2","msg":"event"}
{
  "temp": 42
}
```

If you don't see that, run `./scripts/publish.sh` again and recheck
the logs.

# Now what?

We're not entirely sure! Some of this is just meant to show that our
architecture -- loosely-coupled Knative services swapping cloud events
-- is extensible by the same means in which it's implemented. It's
turtles all the way down!

This article exposes some of the drogue-cloud internals, specifically
that all IoT events currently flow through a single kafka topic. This
is likely to change, of course, partially based on whether users will
even want to introduce their own knative services/functions to extend
the platform. We obviously want to ensure users only receive their own
device events in a multi-tenant environment, for example. So in the
future, we might have topics dedicated to particular users or source
types. Or maybe even use entirely different knative abstractions over
kafka. There's still lots to explore and myriad use cases to support.



[minikube]: https://minikube.sigs.k8s.io/
[kind]: https://kind.sigs.k8s.io/
[podman]: https://podman.io/
[func]: https://github.com/boson-project/func
[kubectl]: https://kubernetes.io/docs/tasks/tools/
[drg]: https://github.com/drogue-iot/drg
[HTTPie]: https://httpie.io/
[CloudEvents]: https://cloudevents.io/
[`kn` plugin]: https://github.com/knative/client/blob/main/docs/README.md
