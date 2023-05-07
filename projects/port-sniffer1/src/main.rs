use clap::Parser;
use clap_num::number_range;
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::sync::{
    mpsc::{channel, Sender},
    Arc,
};
use std::thread;
use std::time::Duration;

fn port_in_range(s: &str) -> Result<u16, String> {
    number_range(s, 1, 65535)
}

#[derive(Parser)]
#[command(
    name = "SimplePortSniffer",
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
    #[arg(long = "timeout", short, default_value = "500")]
    millis_timeout: u64,

    /// Run program on a number of given threads
    #[arg(long, short, default_value = "100")]
    threads: u16,
}

fn main() {
    let args = Arc::new(Args::parse());
    let (tx, rx) = channel::<u16>();

    for i in 0..args.threads {
        let tx = tx.clone();
        let args = Arc::clone(&args);
        let start_port = args.start_port + i;
        thread::spawn(move || scan(args, tx, start_port));
    }

    let mut out = Vec::new();

    drop(tx);
    for recived in rx {
        out.push(recived);
    }

    out.sort();
    println!("Found open ports:");
    for port in out {
        println!("{port}");
    }
}

fn scan(args: Arc<Args>, tx: Sender<u16>, start_port: u16) {
    let mut port = start_port;
    loop {
        let addr = SocketAddr::from((args.ip_address, port));
        match TcpStream::connect_timeout(&addr, Duration::from_millis(args.millis_timeout)) {
            Ok(_) => {
                tx.send(port).unwrap();
            }
            Err(_) => (),
        };

        if (args.end_port - port) <= args.threads {
            break;
        }

        port += args.threads;
    }
}
