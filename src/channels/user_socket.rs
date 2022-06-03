use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::Query;
use axum::response::IntoResponse;
use axum::Extension;
use axum::{routing::get, Router};
use futures::lock::Mutex;
use futures::stream::{SplitSink, StreamExt};
use futures::SinkExt;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::broadcast;
use uuid::Uuid;

#[derive(Debug)]
struct SocketState {
    tx: broadcast::Sender<String>,
}

impl Default for SocketState {
    fn default() -> Self {
        SocketState {
            tx: broadcast::channel(1024).0,
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
struct Socket {
    id: Uuid,
    state: Arc<Mutex<SocketState>>,
    sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    join_ref: Option<String>,
    msg_ref: Option<String>,
    topic: Option<String>,
    channels: HashMap<String, Channel>,
    assigns: Value,
}

impl Socket {
    pub fn new(
        sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
        state: Arc<Mutex<SocketState>>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            state,
            sender,
            join_ref: None,
            msg_ref: None,
            topic: None,
            channels: HashMap::new(),
            assigns: Value::default(),
        }
    }

    fn update(&mut self, message: Value) {
        self.set_join_ref(message[0].clone());
        self.set_msg_ref(message[1].clone());
        self.set_topic(message[2].clone());
    }

    fn topic(&self) -> Option<String> {
        self.topic.clone()
    }

    fn join_channel(&mut self, topic: String) {
        let channel = Channel::new(
            topic.clone(),
            self.join_ref.clone().unwrap(),
            self.msg_ref.clone().unwrap(),
        );
        self.channels.entry(topic).or_insert(channel);
    }

    fn leave_channel(&mut self, topic: String) -> Option<Channel> {
        self.channels.remove(&topic)
    }

    async fn close_channel(&self, channel: Channel) {
        let message = Socket::reply_message(
            Some(channel.join_ref),
            Some(channel.msg_ref),
            Some(channel.topic),
            "phx_close",
            json!({}),
        );
        self.send(message).await;
    }

    async fn push(&self, event: &str, message: Value) {
        let message = Self::reply_message(
            self.join_ref.clone(),
            self.msg_ref.clone(),
            self.topic.clone(),
            event,
            message,
        );
        self.send(message).await;
    }

    async fn boardcast(&self, topic: &str, event: &str, message: Value) {
        self.do_boardcast(None, "boardcast", topic, event, message)
            .await;
    }

    async fn boardcast_from(&self, topic: &str, event: &str, message: Value) {
        self.do_boardcast(
            Some(self.id.to_string()),
            "boardcast_from",
            topic,
            event,
            message,
        )
        .await;
    }

    async fn send(&self, message: Value) {
        let mut sender = self.sender.lock().await;
        if sender
            .send(Message::Text(message.to_string()))
            .await
            .is_err()
        {
            tracing::info!("Error sending message");
        }
    }

    async fn reply_ok(&self, response: Value) {
        self.do_reply("ok", response).await;
    }

    async fn reply_error(&self, response: Value) {
        self.do_reply("error", response).await;
    }

    async fn do_reply(&self, status: &str, response: Value) {
        let message = if response.is_null() {
            json!({"response": {}, "status": status})
        } else {
            json!({"response": response, "status": status})
        };

        self.push("phx_reply", message).await;
    }

    async fn do_boardcast(
        &self,
        from: Option<String>,
        action: &str,
        topic: &str,
        event: &str,
        message: Value,
    ) {
        let message = Self::reply_message(
            self.join_ref.clone(),
            None,
            Some(topic.to_string()),
            event,
            message,
        );

        let data = json!({
            "action": action,
            "from": from,
            "payload": message
        });

        let _ = self.state.lock().await.tx.send(data.to_string());
    }

    fn set_join_ref(&mut self, join_ref: Value) {
        if join_ref.is_null() {
            self.join_ref = None;
        } else if join_ref.is_string() {
            self.join_ref = Some(join_ref.as_str().unwrap().to_string());
        }
    }

    fn set_msg_ref(&mut self, msg_ref: Value) {
        if msg_ref.is_null() {
            self.msg_ref = None;
        } else if msg_ref.is_string() {
            self.msg_ref = Some(msg_ref.as_str().unwrap().to_string());
        }
    }

    fn set_topic(&mut self, topic: Value) {
        if topic.is_null() {
            self.topic = None;
        } else if topic.is_string() {
            self.topic = Some(topic.as_str().unwrap().to_string());
        }
    }

    fn reply_message(
        join_ref: Option<String>,
        msg_ref: Option<String>,
        topic: Option<String>,
        event: &str,
        message: Value,
    ) -> Value {
        json!([
            serde_json::to_value(join_ref).unwrap(),
            serde_json::to_value(msg_ref).unwrap(),
            serde_json::to_value(topic).unwrap(),
            serde_json::to_value(event).unwrap(),
            serde_json::to_value(message).unwrap(),
        ])
    }
}

#[derive(Debug)]
struct Channel {
    topic: String,
    join_ref: String,
    msg_ref: String,
}

impl Channel {
    pub fn new(topic: String, join_ref: String, msg_ref: String) -> Self {
        Self {
            topic,
            join_ref,
            msg_ref,
        }
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Params {
    token: Option<String>,
}

async fn user_socket(
    ws: WebSocketUpgrade,
    Query(params): Query<Params>,
    Extension(state): Extension<Arc<Mutex<SocketState>>>,
) -> impl IntoResponse {
    tracing::info!("params: {:?}", params);
    ws.on_upgrade(|socket| websocket(socket, state))
}

async fn websocket(stream: WebSocket, state: Arc<Mutex<SocketState>>) {
    let (sender, mut receiver) = stream.split();
    let mut rx = state.lock().await.tx.subscribe();

    let sender = Arc::new(Mutex::new(sender));
    let socket = Arc::new(Mutex::new(Socket::new(sender.clone(), state.clone())));
    let rx_socket = socket.clone();

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(message))) = receiver.next().await {
            let message: Value = serde_json::from_str(&message).unwrap();
            tracing::info!("received text: {:?}", message);

            {
                let mut socket = socket.lock().await;
                socket.update(message.clone());
            }

            if message[3].is_string() {
                match message[3].as_str().unwrap() {
                    "heartbeat" => handle_event_heartbeat(socket.clone(), message.clone()).await,
                    "phx_join" => handle_event_phx_join(socket.clone(), message.clone()).await,
                    "phx_leave" => handle_event_phx_leave(socket.clone(), message.clone()).await,
                    _ => handle_event(socket.clone(), message.clone()).await,
                }
            }
        }
    });

    let mut send_task = tokio::spawn(async move {
        while let Ok(message) = rx.recv().await {
            let message: Value = serde_json::from_str(&message).unwrap();
            let action = message["action"].as_str().unwrap();

            {
                let socket = rx_socket.lock().await;

                match action {
                    "boardcast" => socket.send(message["payload"].clone()).await,
                    "boardcast_from" => {
                        let from = message["from"].as_str().unwrap();

                        if from != socket.id.to_string().as_str() {
                            socket.send(message["payload"].clone()).await;
                        }
                    }
                    _ => (),
                }
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }

    // todo: close socket
}

async fn handle_event_heartbeat(socket: Arc<Mutex<Socket>>, message: Value) {
    let topic = message[2].as_str().unwrap();
    let event = message[3].as_str().unwrap();
    if topic == "phoenix" && event == "heartbeat" {
        socket.lock().await.reply_ok(Value::Null).await;
    }
}

async fn handle_event_phx_join(socket: Arc<Mutex<Socket>>, message: Value) {
    let topic = message[2].as_str().unwrap().to_string();
    let mut socket = socket.lock().await;
    socket.join_channel(topic.clone());
    socket.reply_ok(Value::Null).await;
}

async fn handle_event_phx_leave(socket: Arc<Mutex<Socket>>, message: Value) {
    let topic = message[2].as_str().unwrap().to_string();
    let mut socket = socket.lock().await;
    if let Some(channel) = socket.leave_channel(topic) {
        socket.reply_ok(Value::Null).await;
        socket.close_channel(channel).await;
    }
}

async fn handle_event(socket: Arc<Mutex<Socket>>, message: Value) {
    let socket = socket.lock().await;
    let topic = socket.topic().unwrap();

    socket
        .boardcast(topic.as_str(), "boardcast", message[4].clone())
        .await;

    socket
        .boardcast_from(topic.as_str(), "boardcast_from", message[4].clone())
        .await;
}

pub fn router() -> Router {
    Router::new()
        .route("/socket/websocket", get(user_socket))
        .layer(Extension(Arc::new(Mutex::new(SocketState::default()))))
}
