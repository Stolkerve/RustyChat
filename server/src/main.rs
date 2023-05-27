pub mod database;
pub mod handlers;

use tokio::{net::TcpListener, sync::broadcast};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8000")
        .await
        .expect("Couldn't bind server");

    // (Address, To me, encoded msg buffer)
    let (tx, _) = broadcast::channel::<(String, bool, Vec<u8>)>(32);

    let db = database::connect_db().await;
    database::create_tables(&db).await;

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        let tx = tx.clone();
        let rx = tx.subscribe();

        handlers::new_conection(socket, addr.to_string(), tx, rx, db.clone());
    }
}
