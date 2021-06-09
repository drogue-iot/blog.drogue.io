+++
title = "Implement payload mapper with Func and Quarkus"
description = "How to use Func and Quarkus to write real-world serverless functions"
extra.author = "dejanb"
+++

In [the previous post](https://blog.drogue.io/consume-drogue-messages/) we introduced [Func project](https://github.com/boson-project/func) as a very useful framework for writing serverless functions in the Knative world. The post ended with the question what can we do with it? So let's explore one practical use case now.

<!-- more -->

[Earlier](https://blog.drogue.io/digital-twins/), we showed how we can use Eclipse Ditto digital twin platform in combination with Drogue cloud and implemented a service that uses Eclipse Vorto device models to map device telemetry payload to appropriate Ditto commands.

One of the things we continued investigating was how we can use digital twins even without Vorto models for simple use cases. And it all boils down to basic use case of payload mapping.


For example, our device sends telemetry in the simple JSON format like:

```json
{
    "temp": 23.0,
    "hum": 51.0
}
```

and we need to convert it to a Ditto command that will actually modify the state of the digital twin:

```json
{
    "path": "/features",
    "topic": "app_id/simple-thing/things/twin/commands/modify",
    "value": {
        "hum": {
            "properties": {
                "value": "51.0"
            }
        },
        "temp": {
            "properties": {
                "value": 23.0
            }
        }
    }
}
```

This use case is quite common if we want to look at the Drogue cloud as an IoT integration platform. Eclipse Ditto is just one of the services where our telemetry data will end up, but you can imagine various others like time series databases, data lakes, etc. all with their own payload formats.

If you take a look at these kind of services we can see that they are simple, stateless services with well defined input and output. And we have to be able to scale them up and down depending on the data volume. That sounds like a perfect serverless use case.

All this led to the idea of using Func with Quarkus runtime to implement one such service and see what's the developer experience in doing so. So, let's see.

## Cloud native Ditto

Before we dive into implementing the converter service, let's quickly recap Ditto and how it fits into our stack. Ditto provides an HTTP API which can be used to manage your things. In order to start, you need to first create your "thing" using the HTTP API:

```shell
echo "{}" | http --auth ditto:ditto PUT http://$TWIN_API/api/2/things/app_id:simple-thing
```

Beyond HTTP API, Ditto provides a Cloud Events API, which allow us to issue commands (from the example above) as well formed cloud events, like:

```shell
cat modify.json |  http -v --auth ditto:ditto http://$TWIN_API/api/2/cloudevents \
  Content-Type:application/json \
  Ce-Id:9bc19ad9-1c37-4041-b488-5b622e63bbfa \
  Ce-Source:drogue://app%5Fid/simple%2Dthing \
  Ce-Subject:foo \
  Ce-Type:io.drogue.event.v1 \
  Ce-Specversion:1.0 \
  Ce-Application:app \
  Ce-Device:device
```

And we can make sure that our changes took place:

```shell
http --auth ditto:ditto http://$TWIN_API/api/2/things/app_id:simple-thing
HTTP/1.1 200 OK
...
etag: "rev:2"
response-required: false
version: 2

{
    "features": {
        "hum": {
            "properties": {
                "value": "52.0"
            }
        },
        "temp": {
            "properties": {
                "value": 22.5
            }
        }
    },
    "policyId": "app_id:simple-thing",
    "thingId": "app_id:simple-thing"
}
```

Why is this important? Well, it allow us to integrate it into the Knative workflows. Take a look at the following example:

~~~yaml
apiVersion: sources.knative.dev/v1alpha1
kind: KafkaSource
metadata:
  name: ditto-converter-kafka-source
  labels:
    app.kubernetes.io/part-of: digital-twin
spec:
  consumerGroup: ditto-converter
  bootstrapServers:
    - kafka-eventing-kafka-bootstrap.knative-eventing.svc:9092
  topics:
    - knative-messaging-kafka.drogue-iot.iot-channel
  sink:
    ref:
      apiVersion: flows.knative.dev/v1
      kind: Sequence
      name: ditto-converter
---
apiVersion: flows.knative.dev/v1
kind: Sequence
metadata:
  name: ditto-converter
  labels:
    app.kubernetes.io/name: digital-twin
    app.kubernetes.io/part-of: digital-twin
spec:
  channelTemplate:
    apiVersion: messaging.knative.dev/v1alpha1
    kind: KafkaChannel
    spec:
      numPartitions: 1
      replicationFactor: 1
  steps:
    - ref:
        apiVersion: serving.knative.dev/v1
        kind: Service
        name: drogue-ditto-converter
  reply:
    uri: http://ditto:ditto@ditto-nginx.drogue-iot.svc.cluster.local:8080/api/2/cloudevents
~~~

Here, we defined a `KafkaSource` with telemetry messages coming from our devices and wired it to the Knative `Sequence`. The sequence in this case will call our converter service and then post a result to Ditto's Cloud event API (with properly converted payload).

OK, now it all makes more sense and we can start discussing how we are going to implement the converter service.

## Func

We already have a couple Knative services in Drogue cloud implemented with Rust and Quarkus, but they are hand-crafted from the ground up. Now, let's see how Func can help developing these kind of services.

### Developing

You can create a new project with:

```shell
func create --runtime quarkus --trigger events
```

This will create a Quarkus serverless project triggered by Cloud events instead of regular HTTP requests. Just what we need.

You can find a full implementation of our service at <https://github.com/drogue-iot/drogue-ditto-converter>, but let's focus here on some important snippets. The project will use [Quarkus Funqy support](https://quarkus.io/guides/funqy) to provide a stub of our service. Now we can implement it like:

~~~java
    @Funq
    public CloudEvent<DittoCommand> convert(CloudEvent<DrogueDevice> input) throws Exception {

        DrogueDevice device = input.data();
        ...
        DittoCommand command = DittoCommand.from(device);

        CloudEvent<DittoCommand> event = CloudEventBuilder.create(input)
                .build(command);

        return event;
    }
~~~

Here, we provided POJOs for our input and output data formats and everything else (JSON marshalling) will be taken care of. The only thing left to do is to create cloud event and return it.

The nice thing about all this is that we can develop our function in a typical Quarkus fashion without any need for cloud resource. With

```shell
mvn quarkus:dev
```

You'll have live reloading on any code change and you can call your service locally and play around with it.

```shell script
URL=http://localhost:8080/
http -v ${URL} \
  Content-Type:application/json \
  Ce-Id:1 \
  Ce-Source:cloud-event-example \
  Ce-Type:dev.knative.example \
  Ce-Specversion:1.0 \
  Ce-Application:app \
  Ce-Device:device \
  Ce-Dataschema:ditto:simple-thing \
  temp=22.0 \
  hum=52.0
```

There's also a good support for unit and integration testing

~~~java
    @Test
    public void testFunctionIntegration() {
        RestAssured.given().contentType("application/json")
                .body("{\"temp\":23.0}")
                .header("ce-id", "42")
                .header("ce-specversion", "1.0")
                .header("ce-dataschema", "ditto:simple-thing")
                .header("ce-application", "app")
                .header("ce-device", "device")
                .post("/")
                .then().statusCode(200)
                .header("ce-id", notNullValue())
                .header("ce-specversion", equalTo("1.0"))
                .body("topic", equalTo(topic));
    }
~~~

### Deploying

Once you're ready to build your service, `func` CLI have you covered. In Quarkus terms you can build a JVM or native image

```shell script
func build -v                  # build jar
func build --builder native -v # build native binary
```

And the deploy it to your cluster (By default deploy will build image as well).

```shell
func deploy -v
```

This will deploy our function as Knative service, but for full functionality we'll also need our source and sequence deployed:

```shell
kubectl apply -n drogue-iot -f src/main/k8s/ditto-converter.yaml
```
Now we should have everything set, so let's test it out.

### Testing

Let's create a matching app and device in Drogue cloud (in the future we can sync creation of resource in Drogue and Ditto using registry change events).

```shell
drg create app app_id
drg create device --app app_id simple-thing --data '{"credentials": {"credentials":[{ "pass": "foobar" }]}}'
```

And we're ready to send some data

```shell
http --auth simple-thing@app_id:foobar --verify build/certs/endpoints/ca-bundle.pem POST https://$HTTP_ENDPOINT/v1/foo data_schema==ditto:test temp:=21.5 hum=51.0
```

Now we can check the state of our twin and make sure our changes are persisted

```shell
http --auth ditto:ditto http://$TWIN_API/api/2/things/app_id:simple-thing
HTTP/1.1 200 OK
...
etag: "rev:2"
response-required: false
version: 2

{
    "features": {
        "hum": {
            "properties": {
                "value": "51.0"
            }
        },
        "temp": {
            "properties": {
                "value": 21.5
            }
        }
    },
    "policyId": "app_id:simple-thing",
    "thingId": "app_id:simple-thing"
}
```

So we can see that our sensor data went via HTTP endpoint to Knative Kafka channel and then through our payload converter function reached the digital twin platform.

## Future

We hope this post make you realize how Func in combination with Quarkus make a good platform for creating serverless function that have real applications in IoT cloud platforms. We'll strive to make the experience even better in the future. Here are some notes on the experience and future improvements.

First on the Func side, the current function developer workflow is generally good. What would make it even better is making it more tied with the accompanied resources (sources and sequences in our case). We can do two things in that area. We can make `func` CLI manage (deploy/delete) custom yaml resources along with the service itself. But also, it'd be good if `func` would just be able to generate service yaml resource so we can add it to existing workflows (GitOps for example).

On the Quarkus side there are a few small improvements that could be made. Even if the docs says, it requires Java 11, it actually sets Java 8 as default version. Also, the Funq implements its own Cloud Events abstractions, which are not as polished as official SDK. And using the official ones would be a better choice to stay current with the spec implementation in the long run. These are all small niggles that we're gonna feed back into the upstream community and try to improve upon.

And of course, the Rust support for Func is on the way (still in early development). You can take a look at the following PRs ([boson-project/buildpacks#95](https://github.com/boson-project/buildpacks/pull/95) and [boson-project/func#376](https://github.com/boson-project/func/pull/376)) if you'd like to get more information or contribute to the effort. We want to make Rust first-class citizen for developing serverless functions and provide a great experience. And as we want to use them in Drogue cloud we want to help out with the effort. So stay tuned.