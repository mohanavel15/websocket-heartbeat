use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

use std::error::Error;
use std::time::{SystemTime, Duration};

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await.unwrap();

    println!("Listening on: {}", addr);

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream).await {
                println!("Error: {}", e);
            }
        });
    }
}

async fn handle_connection(stream: TcpStream) -> Result<(), Box<dyn Error>> {
    println!("New WebSocket connection: {}", stream.peer_addr()?);
    let ws_stream = accept_async(stream).await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    
    let heart_beat_interval = 5;
    
    let mut message = String::from("{\"op\": 10, \"d\": { \"heartbeat_interval\": ");
    message.push_str((heart_beat_interval * 1000).to_string().as_str());
    message.push_str("}}");
    
    ws_sender.send(Message::Text(message)).await?;

    let mut interval = tokio::time::interval(Duration::from_secs(heart_beat_interval));
    let mut last_heart_beat = SystemTime::now();
    
    loop {
        tokio::select! {
            _ = interval.tick() => {
                let time = SystemTime::now()
                    .duration_since(last_heart_beat)
                    .expect("Time went backwards")
                    .as_secs();
                if time > heart_beat_interval {
                    println!("disconnecting because did not receive heart beat");
                    ws_sender.close().await?;
                    break;
                }
            },
            result = ws_receiver.next() => {
                match result {
                    Some(Ok(message)) => {
                        if message.to_string() == "{\"op\":1}" {
                            last_heart_beat = SystemTime::now();
                            let message = Message::Text(r#"{"op": 11}"#.to_string());
                            ws_sender.send(message).await?;
                        }
                        println!("Message: {:?}", message.to_string())
                    },
                    Some(Err(e)) => {
                        println!("Error: {}", e);
                        break;
                    },
                    None => break,
                }
            }
        }
    }

    Ok(())
}