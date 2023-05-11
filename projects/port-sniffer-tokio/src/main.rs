use clap::Parser;
use clap_num::number_range;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::mpsc::{channel, Sender};
use tokio::time::{timeout, Duration};

fn port_in_range(s: &str) -> Result<u16, String> {
    number_range(s, 1, 65535)
}

#[derive(Parser)]
#[command(
    name = "BestPortSniffer-tokio",
    author = "Dawid <dawidkrasowski05@gmail.com>",
    about = "Simple cli program to check open ports on a given ip address"
)]
struct Args {
    #[arg(long, short, default_value = "1", value_parser=port_in_range)]
    start_port: u16,

    #[arg(long, short, default_value = "65535", value_parser=port_in_range)]
    end_port: u16,

    /// Ip address to check open ports for
    #[arg(long = "ip", short)]
    ip_address: IpAddr,

    /// Time to timeout a tcp connection in milliseconds
    #[arg(long = "timeout", short = 't', default_value = "500")]
    millis_timeout: u64,
}

#[tokio::main]
async fn main() {
    let args = Arc::new(Args::parse());

    let (tx, mut rx) = channel::<u16>(100);

    for port in args.start_port..=args.end_port {
        let tx = tx.clone();
        let args = Arc::clone(&args);

        tokio::spawn(async move {
            scan(args, port, tx).await;
        });
    }

    let mut found_ports = Vec::new();

    drop(tx);
    while let Some(port) = rx.recv().await {
        found_ports.push(port);
    }

    println!("Found open ports:");
    for port in found_ports {
        println!("{port}");
    }
}

async fn scan(args: Arc<Args>, port: u16, tx: Sender<u16>) {
    if let Ok(Ok(_)) = timeout(Duration::from_millis(args.millis_timeout), async {
        TcpStream::connect((args.ip_address, port)).await
    })
    .await
    {
        tx.send(port).await.unwrap();
    }
}
