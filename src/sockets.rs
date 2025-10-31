use serde_json::Value;
use socketioxide::extract::{Data, SocketRef};

pub async fn on_connect(socket: SocketRef, Data(_data): Data<Value>) {
    socket.on("message", |_: SocketRef, Data(payload): Data<Value>| async move {

        println!("message received: {:?}", payload);
    });
}
