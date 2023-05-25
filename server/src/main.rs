pub mod handlers;

use tokio::{
    net::TcpListener, sync::broadcast,
};


#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8000")
        .await
        .expect("Couldn't bind server");

    let (tx, _) = broadcast::channel::<(String, Vec<u8>)>(32);

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        let tx = tx.clone();
        let rx = tx.subscribe();

        handlers::new_conection(socket, addr, tx, rx);
    }
}