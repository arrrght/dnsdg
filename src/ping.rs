extern crate dns_parser;

use dns_parser::{Builder, Packet, ResponseCode};
use dns_parser::{QueryClass, QueryType};
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::time::Instant;

enum SomeError<'a> {
    Conv(std::string::String),
    Io(std::io::Error),
    DnsParser(dns_parser::Error),
    Other(&'a str),
}
impl<'a> std::fmt::Debug for SomeError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            SomeError::Conv(ref err) => write!(f, "Err: {}", err),
            SomeError::Io(ref err) => write!(f, "Err: {}", err),
            SomeError::DnsParser(ref err) => write!(f, "Err: {}", err),
            SomeError::Other(ref err) => write!(f, "Err: {}", err),
        }
    }
}
#[derive(Debug)]
struct Opt<'a> {
    a: &'a str,
    quiet: bool,
    hostname: &'a str,
    server: &'a str,
}

impl<'a> Default for Opt<'a> {
    fn default() -> Opt<'a> {
        Opt {
            a: "String1",
            quiet: false,
            hostname: "google.com",
            server: "8.8.8.8",
        }
    }
}

pub fn dnsping(args: &Vec<String>) {
    let prm = Opt {
        ..Default::default()
    };
    println!("prm: {:?}", prm);
    println!("args: {:?}", args);

    match do_it(prm) {
        Ok(o) => println!("OK: {:?}", o),
        Err(e) => {
            println!("Err: {:?}", e);
            std::process::exit(1);
        }
    }
}

fn prs2(name: &str) -> Result<SocketAddr, SomeError> {
    match name.to_socket_addrs() {
        Err(_) => match format!("{}:53", name).to_socket_addrs() {
            Err(e) => Err(SomeError::Io(e)),
            Ok(o) => o
                .clone()
                .next()
                .ok_or_else(|| SomeError::Other("SockAddr.error")),
        },
        Ok(o) => o
            .clone()
            .next()
            .ok_or_else(|| SomeError::Other("SockAddr.error")),
    }
}

fn do_it(prm: Opt) -> Result<u32, SomeError> {
    let server_sa = prs2(prm.server)?;
    let sock = (match server_sa.is_ipv6() {
        true => UdpSocket::bind("[::]:0"),
        _ => UdpSocket::bind("0.0.0.0:0"),
    }).map_err(SomeError::Io)?;
    sock.connect(server_sa).map_err(SomeError::Io)?;

    let now = Instant::now();
    let mut builder = Builder::new_query(1, true);
    builder.add_question(&prm.hostname, false, QueryType::A, QueryClass::IN);
    let packet = builder.build().unwrap_or_else(|x| x);

    sock.send(&packet).map_err(SomeError::Io)?;
    let mut buf = vec![0u8; 4096];
    sock.recv(&mut buf).map_err(SomeError::Io)?;
    let pkt = Packet::parse(&buf).map_err(SomeError::DnsParser)?;

    if pkt.header.response_code != ResponseCode::NoError
        && pkt.header.response_code != ResponseCode::NameError
    {
        return Err(SomeError::Other("Something bad happening"));
    }

    Ok(now.elapsed().subsec_millis())
}
