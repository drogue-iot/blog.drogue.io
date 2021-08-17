+++
title = "Introducing Websocket integration: another way to consume drogue events"
extra.author = "jbtrystram"
+++

We recently added a websocket endpoint in drogue cloud, to easily get a stream of events for an application, using the well-known websocket protocol.
Let's have a look and build a small demo apps to consume events. 

<!-- more -->

# The Websocket Integration service

The service is pretty straight forward in it's design : it will connect to the appropriate Kafka topic for the application 
and forward any new events in the websocket. The events are pushed in their Cloud-events format, as text messages.

Give it a try ! With drg to get a valid token, opening a socket is as easy as : 
```shell
websocat wss://websocket-integration-drogue-dev.apps.wonderful.iot-playground.org/drogue-public-temperature -H="Authorization: Bearer $(drg whoami -t)"
```
As you can see connecting to a websocket requires authentication for now. The service supports both open ID tokens and API keys.
The above example showed how an openID token can be passed through the `Authorization: Bearer` header.
API keys are used with the `Authorization: Basic` header. Keep on reading for more details. 

The websocket integration service is built on top of the actix actors framework which makes mutlithreading a breeze, 
so you can confidently open multiple connections and don't suffer any slowdowns. 
To give you more flexibility, the `group_id` query parameter enables shared consumption.
Grouping clients will result in the events split evenly between members of a group. The url would look like this:
 ```shell
 wss://websocket-integration.address/yourApp?group_id=myGroup
 ```
If two clients connect using "myGroup" then each will receive every other message. 
If you have an application generating a lot of traffic it's easy to split the load between different consumers.

# Build an app !

A CLI tool is great for testing, but not an ideal base for a consuming application.
Let's build a web socket client using rust and connect it to our websocket integration service. 
As mentioned earlier the service supports authentication with API keys. In the context of a long-running application, 
they are a better fit than open tokens. You can create an API key in the console, under the API > Access keys section.

Now we have everything we need, let's write some code. We'll use the great tungstenite crate for the websocket functionality :
```rust
use anyhow::{anyhow, Context, Result};
use tungstenite::connect;
use tungstenite::http::{header, Request};


pub fn main() -> Result<()> {

    // Here are our connection details
    let url = "wss://websocket-integration-drogue-dev.apps.wonderful.iot-playground.org";
    let application = "drogue-public-temperature";
    let username = "jbtrystram";
    let api_key = "put-your-secret-api-key-here";

    // Preparing the authentication header
    let url = format!("{}/{}", url, application);
    let basic_header = base64::encode(format!("{}:{}", username, api_key));

    let request = Request::builder()
        .uri(url)
        .header(header::AUTHORIZATION, format!("Basic {}", basic_header))
        .body(())?;

    // And connect !
    let (mut socket, response) = connect(request)
         .context("Error connecting to the Websocket endpoint:")?;
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
And voilà, you are streaming events in your application ! This example is pretty basic, you should get fancy and add some async in there.
The dependencies are minimal for this, if you want you can find a buildable cargo project [here]().


# A new tool for drg's arsenal

With the websocket service available we took the opportunity to add a `stream` subcommand to `drg` so you can quickly tap into a stream to do some debugging: 
```
drg stream drogue-public-temperature
```
That's it ! If you have a default app already set in your context, then `drg stream` will fetch the application Id from there.


# What's next ? 

Regarding the websocket integratio there are a couple of features that should be added in a later release: 
- Anonymous authentication, to stream events from public apps without credentials
- Sending commands back to your devices using the websocket other direction
