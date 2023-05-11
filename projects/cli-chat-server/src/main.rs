use std::net::SocketAddr;

use clap::Parser;
use clap_num::number_range;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{self, Receiver, Sender};

fn port_in_range(s: &str) -> Result<u16, String> {
    number_range(s, 1, 65535)
}

#[derive(Parser)]
struct Args {
    /// Specify the port for the server to listen on
    #[arg(short, long, default_value = "8080", value_parser=port_in_range)]
    port: u16,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let listener = TcpListener::bind(("localhost", args.port)).await.unwrap();
    println!("Listening on localhost:{}", args.port);

    let (tx, _rx) = broadcast::channel::<String>(16);
    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                println!("Client connected: {}", addr);
                let tx = tx.clone();
                let rx = tx.subscribe();

                tokio::spawn(async move {
                    handle_connection(socket, addr, tx, rx).await;
                });
            }
            Err(e) => {
                println!("Couldn't connect client: {}", e);
            }
        };
    }
}

async fn handle_connection(
    mut socket: TcpStream,
    addr: SocketAddr,
    tx: Sender<String>,
    mut rx: Receiver<String>,
) {
    let (reader, mut writer) = socket.split();

    let mut buff = BufReader::new(reader);
    let mut line = String::new();

    loop {
        tokio::select! {
            result = buff.read_line(&mut line) => {
                if result.unwrap() == 0 {
                    println!("Client disconnected: {}", addr);
                    break;
                }
                tx.send(line.clone()).unwrap();
                line.clear();
            }
            result = rx.recv() => {
                writer.write_all(result.unwrap().as_bytes()).await.unwrap();
            }
        }
    }
}
