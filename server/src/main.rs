use axum::routing::get;
use socketioxide::{
    extract::{AckSender, Data, SocketRef},
    SocketIo,
};

async fn on_connect(socket: SocketRef, Data(data): Data<serde_json::Value>) {
    socket.on(
        "event",
        async |socket: SocketRef, Data::<serde_json::Value>(data)| {
            let event = serde_json::from_value::<netcode::Event>(data);

            let event = match event {
                Ok(e) => e,
                Err(err) => {
                    socket.emit(
                        "error",
                        &format!("Error while parsing event payload: {}", err),
                    );
                    return;
                }
            };

            // socket.emit("message-back", &data).ok();
        },
    );

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
