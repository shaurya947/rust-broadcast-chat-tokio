use futures_util::sink::SinkExt;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{
    broadcast::{channel, Sender},
    Mutex,
};
use tokio_websockets::{Message, ServerBuilder, WebsocketStream};

async fn handle_connection(
    _addr: SocketAddr,
    ws_stream: WebsocketStream<TcpStream>,
    bcast_tx: Sender<String>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let ws_stream = Arc::new(Mutex::new(ws_stream));
    let ws_stream_1 = Arc::clone(&ws_stream);

    let recv_message_future = async {
        let msg = ws_stream.lock().await.next().await;
        if let Some(Ok(msg)) = msg {
            bcast_tx.send(msg.as_text()?.to_string())?;
            Ok::<(), Box<dyn Error + Send + Sync>>(())
        } else {
            Err(Box::<dyn Error + Send + Sync>::from(
                "Error receiving message",
            ))
        }
    };
    tokio::pin!(recv_message_future);

    let mut bcast_rx = bcast_tx.subscribe();
    let bcast_message_future = async {
        let message = bcast_rx.recv().await?;
        ws_stream_1
            .lock()
            .await
            .send(Message::text(message))
            .await?;
        Ok::<(), Box<dyn Error + Send + Sync>>(())
    };
    tokio::pin!(bcast_message_future);

    loop {
        tokio::select! {
            res = &mut recv_message_future => res,
            res = &mut bcast_message_future => res
        }?
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (bcast_tx, _) = channel(16);

    let listener = TcpListener::bind("127.0.0.1:2000").await?;
    println!("listening on port 2000");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from {addr:?}");
        let bcast_tx = bcast_tx.clone();
        tokio::spawn(async move {
            // Wrap the raw TCP stream into a websocket.
            let ws_stream = ServerBuilder::new().accept(socket).await?;

            handle_connection(addr, ws_stream, bcast_tx).await
        });
    }
}
