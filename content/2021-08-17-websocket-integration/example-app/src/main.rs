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