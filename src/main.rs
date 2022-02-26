use futures::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::Message};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("Server couldnt be created");

    loop {
        let (stream, _) = listener.accept().await.expect("Couldnt accept connection");

        let websocket = accept_async(stream)
            .await
            .expect("Couldnt create websocket");

        println!("WebSocket created");

        let (mut sink, mut stream) = websocket.split();

        tokio::spawn(async move {
            tokio::spawn(async move {
                sink.feed(Message::Text("From server".to_string()))
                    .await
                    .expect("Couldn't send message");

                if sink.flush().await.is_err() {
                    eprintln!("Closed by err");
                }
            });

            let stream_future = tokio::spawn(async move {
                while let Some(Ok(message)) = stream.next().await {
                    println!("{:?}", message);
                }
            });

            let _ = stream_future.await;

            println!("WebSocket connection closed");
        });
    }
}
