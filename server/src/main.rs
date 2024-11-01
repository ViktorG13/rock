use anyhow::{anyhow, Result};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::Mutex,
};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message, WebSocketStream};

type Clients = Arc<Mutex<Vec<Arc<Mutex<WebSocketStream<TcpStream>>>>>>;

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "127.0.0.1:3000".to_string();
    let listener = TcpListener::bind(&addr).await?;
    println!("WebSocket server started on ws://{addr}");

    let clients: Clients = Arc::new(Mutex::new(vec![]));

    while let Ok((stream, _)) = listener.accept().await {
        let clients_clone = Arc::clone(&clients);
        tokio::spawn(handle_connection(stream, clients_clone));
    }

    Ok(())
}

async fn handle_connection(stream: TcpStream, clients: Clients) -> Result<()> {
    let ws_stream = accept_async(stream).await?;
    println!("WebSocket Connection established");

    let ws_stream = Arc::new(Mutex::new(ws_stream));
    {
        let mut clients = clients.lock().await;
        clients.push(ws_stream.clone());
    }

    send_message(&ws_stream, "Welcome to the Rock".to_string()).await?;

    let ws_stream_clone = ws_stream.clone();
    let client_clone = Arc::clone(&clients);

    tokio::spawn(async move {
        while let Some(msg) = ws_stream_clone.lock().await.next().await {
            let msg = match msg {
                Ok(msg) => msg,
                Err(e) => {
                    eprintln!("Error while receiving message: {e:?}");
                    break;
                }
            };

            if msg.is_text() {
                let received_text = msg.to_text().unwrap_or_default();
                println!("Mensagem recebida: {received_text}");

                let mut disconnected_clients = vec![];

                let clients = client_clone.lock().await;
                for client in clients.iter() {
                    if !Arc::ptr_eq(client, &ws_stream_clone) {
                        if let Err(e) = send_message(client, received_text.to_string()).await {
                            eprintln!("Failed to send message: {e:?}");
                            disconnected_clients.push(client.clone());
                        }
                    }
                }

                let mut clients = client_clone.lock().await;
                clients.retain(|client| !disconnected_clients.iter().any(|dc| Arc::ptr_eq(client, dc)));
            }
        }
        println!("WebSocket connection closed");
    });
    
    Ok(())
}

async fn send_message(
    ws_stream: &Arc<Mutex<WebSocketStream<TcpStream>>>,
    text: String,
) -> Result<()> {
    let mut stream = ws_stream.lock().await;
    stream
        .send(Message::Text(text))
        .await
        .map_err(|e| anyhow!("Failed to send message: {e}"))?;
    Ok(())
}
