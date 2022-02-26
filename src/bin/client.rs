use websockets::WebSocketClient;

#[tokio::main]
async fn main() {
    let mut web_socket_client = WebSocketClient::new("127.0.0.1:8080").await.unwrap();

    web_socket_client
        .on_message(|message| async move { println!("{}", message) })
        .unwrap();

    web_socket_client
        .send("Siema eniu".to_string())
        .await
        .unwrap();

    web_socket_client.close().await.unwrap();
}
