use axum::routing::get;
use socketioxide::{
    extract::{AckSender, Data, SocketRef},
    SocketIo,
};

async fn on_connect(socket: SocketRef, Data(data): Data<serde_json::Value>) {
    socket.emit("auth", &data).ok();

    socket.on("message", async |socket: SocketRef, Data::<serde_json::Value>(data)| {
        socket.emit("message-back", &data).ok();
    });

    socket.on(
        "message-with-ack",
        async |Data::<serde_json::Value>(data), ack: AckSender| {
            ack.send(&data).ok();
        },
    );
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (layer, io) = SocketIo::new_layer();

    io.ns("/", on_connect);

    let app = axum::Router::new().layer(layer);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7878").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}