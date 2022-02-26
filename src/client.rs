use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt, Future};
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{client_async, WebSocketStream};

#[derive(Debug)]
pub enum WebSocketClientError {
    ConnectionClosed,
    OnMessageAlreadyExists,
    WebSockerClosed,
    ServerNotResponding,
    WebSocketNotSupported,
}

pub struct WebSocketClient {
    sink: SplitSink<WebSocketStream<TcpStream>, Message>,
    stream: Option<SplitStream<WebSocketStream<TcpStream>>>,
    stream_join_handle: Option<JoinHandle<()>>,
    closed: bool,
}

impl WebSocketClient {
    pub async fn new(address: &str) -> Result<WebSocketClient, WebSocketClientError> {
        let stream = TcpStream::connect(address)
            .await
            .map_err(|_| WebSocketClientError::ServerNotResponding)?;

        let (websocket, _) = client_async(format!("ws://{}", address), stream)
            .await
            .map_err(|_| WebSocketClientError::WebSocketNotSupported)?;

        let (sink, stream) = websocket.split();

        Ok(WebSocketClient {
            sink,
            stream: Some(stream),
            stream_join_handle: None,
            closed: false,
        })
    }

    pub async fn send(&mut self, message: String) -> Result<(), WebSocketClientError> {
        self.check_closed()?;

        self.sink
            .send(Message::Text(message))
            .await
            .map_err(|_| WebSocketClientError::ConnectionClosed)?;

        Ok(())
    }

    pub fn on_message<F, Fu>(&mut self, mut process_message: F) -> Result<(), WebSocketClientError>
    where 
        Fu: Future<Output = ()> + Send,
        F: FnMut(String) -> Fu + Send + 'static,
    {
        self.check_closed()?;

        let mut stream = self
            .stream
            .take()
            .ok_or_else(|| WebSocketClientError::OnMessageAlreadyExists)?;

        let stream_join_handle = tokio::spawn(async move {
            while let Some(Ok(m)) = stream.next().await {
                if let Message::Text(message) = m {
                    process_message(message).await;
                }
            }
        });

        self.stream_join_handle = Some(stream_join_handle);

        Ok(())
    }

    pub async fn close(&mut self) -> Result<(), WebSocketClientError> {
        self.check_closed()?;

        self.sink
            .send(Message::Close(None))
            .await
            .map_err(|_| WebSocketClientError::ConnectionClosed)?;

        if let Some(stream_join_handle) = self.stream_join_handle.take() {
            stream_join_handle
                .await
                .map_err(|_| WebSocketClientError::ConnectionClosed)?;
        }

        self.closed = true;

        Ok(())
    }

    fn check_closed(&self) -> Result<(), WebSocketClientError> {
        match self.closed {
            true => Err(WebSocketClientError::WebSockerClosed),
            false => Ok(()),
        }
    }
}
