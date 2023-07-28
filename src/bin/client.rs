use std::println;

use futures_util::SinkExt;
use http::Uri;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_websockets::{ClientBuilder, Message};

#[tokio::main]
async fn main() -> Result<(), tokio_websockets::Error> {
    let mut ws_stream = ClientBuilder::from_uri(Uri::from_static("ws://127.0.0.1:2000"))
        .connect()
        .await?;

    let stdin = tokio::io::stdin();
    let mut stdin = BufReader::new(stdin).lines();

    loop {
        tokio::select! {
            msg_received = ws_stream.next() => {
                match msg_received {
                    Some(Ok(msg)) => {
                        println!("Message received: {}", msg.as_text()?);
                        Ok(())
                    }
                    Some(Err(e)) => Err(e),
                    None => Ok(())
                }
            }
            user_input = stdin.next_line() => {
                match user_input {
                    Ok(Some(input)) => {
                        ws_stream.send(Message::text(input)).await?;
                        Ok(())
                    }
                    Ok(None) => Ok(()),
                    Err(e) => Err(e.into())
                }
            }
        }?
    }
}
