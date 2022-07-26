use std::mem::size_of;
use std::time::Duration;
use std::{fs, thread};

use lazy_static::lazy_static;
use tokio::sync::broadcast::{channel, Receiver, Sender};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

// use crate::logic::Game;
// use crate::net::SnakeEvent;

use rusty_snake::{Game, Signal, SnakeEventType};
use rusty_snake::SnakeEvent;

// Own Modules
// mod logic;
// mod net;

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

lazy_static! {
    pub static ref CONF: toml::Value =
        toml::from_str(&fs::read_to_string("Server.toml").unwrap()).unwrap();
    pub static ref SERVER_ADDRESS: String = CONF["server_address"].as_str().unwrap().to_string();
}

#[tokio::main]
async fn main() {
    println!("Server starting...");
    let (tx_game, _rx_game) = channel::<Game>(32);
    let (tx_event, _rx_event) = channel::<SnakeEvent>(32);

    let tx_game_copy = tx_game.clone();
    let tx_event_copy = tx_event.clone();
    thread::Builder::new()
        .name("Server Thread".to_string())
        .spawn(move || server(tx_game_copy, tx_event_copy.subscribe()))
        .unwrap();

    let listener = TcpListener::bind(SERVER_ADDRESS.to_string()).await.unwrap();

    println!("Listening on: {}", listener.local_addr().unwrap());

    loop {
        let (mut socket, _addr) = listener.accept().await.unwrap();
        println!("Got new connection: {}", _addr.clone());
        let mut rx_game = tx_game.subscribe();
        let tx_event = tx_event.clone();
        tokio::spawn(async move {
            loop {
                let mut buf_len = [0u8; size_of::<usize>()];
                tokio::select! {
                    Ok(g) = rx_game.recv() => {
                        let serialized_data = bincode::serialize(&g).unwrap();
                        let serializes_len = bincode::serialize(&serialized_data.len()).unwrap();
                        let mut buf = Vec::<u8>::new();
                        buf.extend(&serializes_len);
                        buf.extend(&serialized_data);
                        socket.write_all(&buf).await.unwrap();
                    },
                    Ok(bytes) = socket.read_exact(&mut buf_len) => {
                        if bytes == 0 {
                            println!("User Disconnected (Lost Connection): {}", _addr.clone());
                            break;
                        }
                        let packet_len = bincode::deserialize(&buf_len).unwrap();
                        let mut buf = vec![0u8; packet_len];
                        socket.read_exact(&mut buf).await.unwrap();
                        let event: SnakeEvent = bincode::deserialize(&buf).unwrap();
                        if let SnakeEventType::Signal(Signal::Disconnect) = event.event_type {
                            tx_event.send(event).unwrap();
                            println!("User Disconnected (Event): {}", _addr.clone());
                            break;
                        }
                        tx_event.send(event).unwrap();
                    }
                };
            }
            println!("Exiting {} task", _addr.clone());
        });
    }
}
