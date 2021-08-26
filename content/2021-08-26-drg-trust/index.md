+++
title = "Using X509 Client certificates for devices authentication with drogue-cloud"
extra.author = "jb_and_vedang"
+++

This article shows how you can setup X.509 certificates to authenticate devices connecting to Drogue Cloud and an example of how [`drg`](https://github.com/drogue-iot/drg) makes it easy.

<!-- more -->

# X509 Authentication?

If you are already know how chains of trust works with X509 certs feel free to skip to the next part. Here is a quick explanation otherwise:

Before delving into certificates, you need to know they rely on *asymmetric cryptography*. In short, asymmetric cryptography is a technique that enables encryption and decryption of data with two keys.
Let's take a key pair (A, B): stuff encrypted with key A can only be decrypted with key B, and vice versa. In the certificates implementation of this technique, we name this key pair (Public, Private).

You already use X509 and trust anchors in your daily life, right on this very blog! It's the famous green padlock showing you can trust the connection is secure.
But how could you trust that, if that is the first time you visit this website ?

The security relies on a shared trusted entity. It's the same mechanism that you use when you accept bank notes from someone, you know the value is real because you trust the bank that created them.
In the X509 spec, the common entity that both parties trust is called a *trust anchor*. There is a nice twist: x509 certificates are harder to counterfeit than banknotes!

When you connect to `blog.drogue.io`, the server presents to your browser a certificate containing its public key. This certificate is *signed* using a trust anchor's private key.
As you already know and trust this trust anchor, you can verify the certificate by using the trust anchor's public key. 
Once you verified the certificate, you can trust it, because the anchor said it's worthy.

That's the jist of it! X509 has a lot of other details to know about, but that overview is enough for now. 

# Client-side certificates

Drogue-cloud supports authenticating devices using client certificates.
The use of X.509 certificates simplifies the process of device authentication, providing a one-to-many relationship between an application and its devices.
X.509 certificates provide stronger client authentication compared to access tokens or username-password combinations because the private key never leaves the device.

## Client certificates 

The authentication flow using a client-side certificate is the same as explained earlier: the device, when connecting to drogue-cloud, presents a certificate. 
Drogue-cloud then verifies the signature on the certificate to authenticate the device.

## Application trust anchor

To be able to verify the certificate, we need to set up a common trust anchor between the device and Drogue cloud.
We will do that by generating a public/private key pair. This key pair is used to sign a certificate. This _**self-signed**_ certificate is then attached to an application. 

Now the private key of the application can be used to sign certificates of devices, which will then be presented at the time of authentication.

# Setting things up 

In this example we will use our Drogue command line tool, `drg`, as it makes some steps easier. 
See how to install `drg` and login to a Drogue Cloud instance [here](https://github.com/drogue-iot/drg#installation).
To know more about how to use `drg` for managing apps and devices, you can check out this [blog](https://blog.drogue.io/drg-guided-tour/).

**DISCLAIMER**: Some features used in this walkthrough are not yet released. To reproduce them you'll need to install `drg` from source and deploy drogue-cloud from the latest available images.
You can try to use our [development deployment](https://console-drogue-dev.apps.wonderful.iot-playground.org/), at your own risk! 

We will also show how to set up a trust anchor manually.

You will need a running drogue-cloud instance, obviously! You can use the [sandbox](https://sandbox.drogue.cloud/) we provide.

## Add the application trust anchor

If you don't already have one, let's create an application and call it `demo-app`:
```
drg create app demo-app
```

Now, we need to create a keypair and a self-signed certificate to add a new trust anchor (a.k.a. CA certificate) to the application object.
We also specify a 'key-output' to save the private key on our system. This key file is important, as it will be used later to sign device certificates.

```
drg trust create demo-app --key-output demo-app-key.pem
```

Under the hood, drg does 3 steps: 
- Create a key pair for the application. 
- Generate and sign a certificate with thoses keys
- Upload the serialized (PEM format) certificate in the spec section of the application, base64 encoded.


## Generate a certificate for the device

Then we will set up a certificate for a device within this application. If necessary, you can create one called `demo-device`:
```
drg create device demo-device --app demo-app
```

Now the last thing we need is to generate a certificate for the device, and then sign it using our application trust anchor:
```
drg trust enroll demo-device  --app demo-app --ca-key demo-app-key.pem --out dcert.pem --key-output dkey.pem
```
Let's break down all the arguments used here:
- `--app` : the application to use as a trust anchor. The device must belong to that app.
- `--device`: the device ID of the device we want a certificate for.
- `--ca-key`: the private key of the app trust anchor, used to sign the device's certificate.
- `--out`: the output file to save the signed device certificate and public key. 
- `--key-output`: the output file to save the device private key.

Here, `dcert.pem` will contain the certificate the device will be presenting at the time of authentication.


### Device name alias 

`drg` handles the certificate generation, signing, and also does an extra step: it adds an alias for the device. 
In order to successfully authenticate, a device must have a name matching its certificate's "SUBJECT-DN" field.
The default (when created by `drg`) looks like this: `CN=<deviceId>, O=Drogue IoT, OU=<appId>`. This is not a great device name when it comes to management, so we use an alias here. 
You can add custom aliases for devices with drg: `drg set alias <deviceID> <newAlias>`. Note that aliases can be used interchangeably for device authentication, but not for device management.


## Using the certificate

The process is now complete and we can use the freshly signed certificate and private key to authenticate our device to Drogue Cloud! 

To simulate the process of a device sending some data to the cloud over MQTT, let's use the [MQTT CLI](https://github.com/hivemq/mqtt-cli/releases/tag/v4.6.4) tool:

```
mqtt -h <host> -p <port> --cert dcert.pem --key dkey.pem -t <topic> -m <message>
```

Hurray! Your message is published, the device was sucessfully authenticated.


# Bonus features

`drg` packs a few more features to give you more flexibility.

## Changing the signature algorithm

The signature algorithm used to sign the app and device certificates is by default set to EdDSA, but it can be changed by specifying the `--algo` parameter. 
The supported signature algorithms are RSA, ECDSA, and EdDSA. An example of using RSA:

```
drg trust create --app demo-app --algo RSA --key-output demo-app-key.pem
```

You can also change the default signature algorithm for your [context](https://github.com/drogue-iot/drg#configuration-file):

```
drg context set-default-algo RSA
```

## Using existing key pairs

You may have an existing key pair that you would like to use for the App or device certificate. This can be done easily with the `--key-input` parameter. 
The private key needs to be in [PKCS8](https://en.wikipedia.org/wiki/PKCS_8) format to be parsed by `drg`. Let's look at an example to generate a key using OpenSSL.

To generate RSA key: `openssl genrsa -out key.pem 2048`

To convert this key into the required PKCS8 format:

```
openssl pkcs8 -topk8 -nocrypt -outform der -on key.pem > private.pk8
```

Now, it is ready to be supplied to `drg`

```
drg trust create --app demo-app --key-input private.pk8
```

If there are some more bonus features that you would like to see in drg please let us know by opening a [request](https://github.com/drogue-iot/drg/issues)!


## Alternative: do everything manually with OpenSSL 
If you like getting your hands dirty with OpenSSL, here are the steps to do all that manually: 

Generate the certificate for the application :
```
openssl req -x509 -nodes -newkey rsa:4096 -keyout app-key.pem -out app-cert.pem -days 365 -subj "/O=Drogue IoT/OU=Cloud/CN=demo-app"
```

Now upload the application certificate to drogue cloud (`drg edit application demo-app`). The spec should look like this: 
```json
"spec" : {
      "trustAnchors": {
                   "anchors": [ 
                          { "certificate": "<cert content goes here>"} 
                          ]
                   }
        }
}
```
The `certificate` key must contain the whole certificate serialized as PEM.

Then generate a certificate signing request for the device and sign it
```
openssl req -nodes -newkey rsa:4096 -keyout device-private.pem -days 365 -subj "/O=Drogue IoT/OU=app1/CN=device" > device-cert.req
cat device-cert.req | openssl x509 -req -extfile ca.cnf -extensions "san_ext" -out device.crt -days 3650 -CA app-cert.pem -CAkey app-key.pem -set_serial 2
```

With the `ca.cnf` file as follows: 
```
[san_ext]

extendedKeyUsage = serverAuth, clientAuth
```
Don't forget to create an alias entry for your device as explained in the "Device name alias" paragraph, and you should be good to go!

## Special thanks

Thanks to Vedang who did all the implementation work to make certificates easy with drg. This work was done as a Google Summer of Code project. 
You can read his writeup [on his personnal blog](https://vedangj044.github.io/blog/gsoc-phase2/).

