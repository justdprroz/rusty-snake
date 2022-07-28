use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

#[tokio::main]
async fn main() {
    println!("Server started...");
    let listener = TcpListener::bind("127.0.0.1:42069").await.unwrap();
    loop {
        let (mut socket, _addr) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            loop {
                let mut buffer = [0u8; 1024];
                let bytes_read = socket.read(&mut buffer).await.unwrap();
                if bytes_read == 0 {
                    break;
                }
                socket.write_all(&buffer[..bytes_read]).await.unwrap();
            }
        });
    }
}
