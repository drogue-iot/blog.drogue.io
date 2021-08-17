+++
title = "Introducing Websocket integration: another way to consume drogue events"
extra.author = "jbtrystram"
+++

We recently added a websocket endpoint in drogue cloud, to easily get a stream of events for an application, using the well-known websocket protocol.
Let's have a look and build a small demo apps to consume events. 

<!-- more -->

# The Websocket Integration service

The service is pretty straight forward in its design : it will connect to the appropriate Kafka topic for the application 
and forward any new events in the websocket. The events are pushed in their Cloud-events format, as text messages.

Give it a try ! Opening a socket is as easy as : 
```shell
websocat wss://websocket-integration-drogue-dev.apps.wonderful.iot-playground.org/drogue-public-temperature
```

## Authentication

As you can see we used authentication to open the websocket. The service accepts open ID tokens, API keys and nothing/anonymous (if your app is configured for public access).
Authorization requests are made with a "read" permission.

An openID token can be passed through the `Authorization: Bearer` header. Here using `drg` to get a token : 
``` 
websocat wss://websocket-integration.address/yourApplication -H="Authorization: Bearer $(drg whoami -t)"
```

For API keys you need to use the `Basic` header, where the password will be the API key. The header should look like this: 
```
Authorization: Basic base64(your-username:your-api-key)
```

## Shared consumption
The websocket integration service is built on top of the actix actors framework which makes mutlithreading a breeze, 
so you can confidently open multiple connections and don't suffer any slowdowns. \
To give you more flexibility, the `group_id` query parameter enables shared consumption.
Grouping clients will result in the events split evenly between members of a group. \
The url would look like this:
 ```shell
 wss://websocket-integration.address/yourApp?group_id=myGroup
 ```
If two clients connect using "myGroup" then each will receive every other message. 
If you have an application generating a lot of traffic it's easy to split the load between different consumers.

# Build an app !

A CLI tool is great for testing, but not an ideal base for a consuming application.
Let's build a small application using rust and connect it to our websocket integration service. 
This example connects to a public application, so there is no need for credentials.

Now we have everything we need, let's write some code. We'll use the great tungstenite crate for the websocket functionality :
```rust
use anyhow::{anyhow, Context, Result};
use tungstenite::connect;
use tungstenite::http::{header, Request};
use url::Url;

pub fn main() -> Result<()> {
    // Here are our connection details
    let url = "wss://websocket-integration-drogue-dev.apps.wonderful.iot-playground.org";
    let application = "drogue-public-temperature";
    let url = format!("{}/{}", url, application);

    // And connect !
    let (mut socket, response) = connect(Url::parse(url).unwrap()).context("Error connecting to the Websocket endpoint:")?;
    println!("Connected to websocket");
    println!("HTTP response code: {}", response.status());

    // Now we can simply poll the connection for new messages.
    loop {
        let msg = socket.read_message();
        match msg {
            Ok(m) => {
                // ignore protocol messages, only show text
                if m.is_text() {
                    println!("{}", m.into_text().expect("Invalid message"));
                }
            }
            Err(e) => break Err(anyhow!(e)),
        }
    }
}
```
And voilà, you are streaming events in your application ! This example is pretty basic, you should get fancy and add some async and authentication in there. \

In the context of a long-running application, API key are a better fit than openID tokens. You can create an API key in the console, under the API > Access keys section.
You can find a buildable cargo project that shows how to use the API token authentication [here](example-app/).


# A new tool for drg's arsenal

With the websocket service available we took the opportunity to add a `stream` subcommand to `drg` so you can quickly tap into a stream to do some debugging: 
```
drg stream drogue-public-temperature
```
That's it ! If you have a default app already set in your context, then `drg stream` will fetch the application Id from there.


# What's next ? 

The next stop is to be able to send commands back to your devices using the websocket other direction.
Expect to see it in a later release ! 
