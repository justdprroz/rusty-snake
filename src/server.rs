use std::thread;
use std::time::Duration;

use tokio::sync::broadcast::{channel, Receiver, Sender};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

use crate::logic::Game;
use crate::net::SnakeEvent;

// Own Modules
mod logic;
mod net;

fn server(tx: Sender<Game>, mut rx: Receiver<SnakeEvent>) {
    let (gw, gh) = (40, 20);
    let mut game: Game = Game::new(gw, gh, 10, 5, true);
    game.add_missing_food();
    loop {
        loop {
            match rx.try_recv() {
                Ok(event) => game.handle_events(event.event_type, event.event_owner),
                Err(_) => break,
            }
        }
        game.step();
        game.add_missing_food();
        tx.send(game.clone()).unwrap();
        thread::sleep(Duration::from_millis(100));
    }
}

#[tokio::main]
async fn main() {
    println!("Server started...");
    let (tx_game, _rx_game) = channel::<Game>(32);
    let (tx_event, _rx_event) = channel::<SnakeEvent>(32);

    let tx_game_copy = tx_game.clone();
    let tx_event_copy = tx_event.clone();
    thread::Builder::new()
        .name("Server Thread".to_string())
        .spawn(move || server(tx_game_copy, tx_event_copy.subscribe()))
        .unwrap();

    let listener = TcpListener::bind("127.0.0.1:42069").await.unwrap();
    loop {
        let (mut socket, _addr) = listener.accept().await.unwrap();
        println!("Got new connection");
        let mut rx_game = tx_game.subscribe();
        let tx_event = tx_event.clone();
        tokio::spawn(async move {
            loop {
                let mut buf = [0u8; 1024];
                if let Ok(g) = rx_game.try_recv() {
                    let buf = bincode::serialize(&g).unwrap();
                    socket.write_all(&buf).await.unwrap();
                }
                if let Ok(_) = socket.try_read(&mut buf) {
                    let event: SnakeEvent = bincode::deserialize(&buf).unwrap();
                    tx_event.send(event).unwrap();
                };
            }
        });
    }
}
