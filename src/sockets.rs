use serde::{Deserialize, Serialize};
use serde_json::Value;
use socketioxide::extract::{Data, SocketRef};

#[derive(Deserialize, Debug)]
struct SubscribePayload {
    market_id: String,
}

pub async fn on_connect(socket: SocketRef, Data(_data): Data<Value>) {
    println!("Socket connected: {:?}", socket.id);

    socket.on("subscribe:orderbook", |socket: SocketRef, Data(payload): Data<SubscribePayload>| async move {
        let room = format!("orderbook:{}", payload.market_id);
        socket.join(room.clone());
        println!("Socket {} joined room {}", socket.id, room);
    });

    socket.on("unsubscribe:orderbook", |socket: SocketRef, Data(payload): Data<SubscribePayload>| async move {
        let room = format!("orderbook:{}", payload.market_id);
        socket.leave(room.clone());
        println!("Socket {} left room {}", socket.id, room);
    });

    socket.on("subscribe:trades", |socket: SocketRef, Data(payload): Data<SubscribePayload>| async move {
        let room = format!("trades:{}", payload.market_id);
        socket.join(room.clone());
        println!("Socket {} joined room {}", socket.id, room);
    });

    socket.on("unsubscribe:trades", |socket: SocketRef, Data(payload): Data<SubscribePayload>| async move {
        let room = format!("trades:{}", payload.market_id);
        socket.leave(room.clone());
        println!("Socket {} left room {}", socket.id, room);
    });

    socket.on("subscribe:timeseries", |socket: SocketRef, Data(payload): Data<SubscribePayload>| async move {
        let room = format!("timeseries:{}", payload.market_id);
        socket.join(room.clone());
        println!("Socket {} joined room {}", socket.id, room);
    });

    socket.on("unsubscribe:timeseries", |socket: SocketRef, Data(payload): Data<SubscribePayload>| async move {
        let room = format!("timeseries:{}", payload.market_id);
        socket.leave(room.clone());
        println!("Socket {} left room {}", socket.id, room);
    });

    socket.on("message", |_: SocketRef, Data(payload): Data<Value>| async move {
        println!("message received: {:?}", payload);
    });
}
