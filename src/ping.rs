extern crate clap;
extern crate dns_parser;

use clap::{value_t, ArgMatches};
use dns_parser::{Builder, Packet, ResponseCode};
use dns_parser::{QueryClass, QueryType};
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::time::Instant;

enum SomeError<'a> {
    Io(std::io::Error),
    DnsParser(dns_parser::Error),
    Other(&'a str),
}

impl<'a> std::fmt::Debug for SomeError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            SomeError::Io(ref err) => write!(f, "ErrIO: {}", err),
            SomeError::DnsParser(ref err) => write!(f, "ErrDNSParser: {}", err),
            SomeError::Other(ref err) => write!(f, "ErrOther: {}", err),
        }
    }
}
#[derive(Debug, Copy, Clone)]
struct Opt<'a> {
    hostname: &'a str,
    server: &'a str,
    count: u32,
    interval: u64
}

pub fn dnsping(args: &ArgMatches) {
    let prm = Opt {
        server: &value_t!(args, "server", String).unwrap(),
        hostname: &value_t!(args, "hostname", String).unwrap(),
        count: value_t!(args, "count", u32).unwrap(),
        interval: value_t!(args, "interval", u64).unwrap(),
    };

    let mut results: Vec<u32> = (0..prm.count)
        .map(|c| match do_it(prm) {
            Ok(o) => {
                println!(
                    "{} bytes from {}: seq={:<3} time={:.3} ms ",
                    o.len,
                    prm.server,
                    c,
                    o.time as f32 / 1000.0
                );
                std::thread::sleep(std::time::Duration::from_secs(prm.interval));
                o.time
            }
            Err(e) => {
                println!("Err: {:?}", e);
                std::process::exit(1);
            }
        }).collect();
    println!("\nresults: {:?}", results);
    results.sort();
    let max = results.last().unwrap();
    let min = results.first().unwrap();
    let sum: u32 = results.iter().sum();
    let aver: f32 = sum as f32 / results.len() as f32;
    println!("min: {}, max: {}, avg: {}", min, max, aver);
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
#[derive(Debug)]
struct Ans {
    time: u32,
    len: usize,
}
fn do_it(prm: Opt) -> Result<Ans, SomeError> {
    let server_sa = prs2(prm.server)?;
    let sock = (match server_sa.is_ipv6() {
        true => UdpSocket::bind("[::]:0"),
        _ => UdpSocket::bind("0.0.0.0:0"),
    }).map_err(SomeError::Io)?;
    sock.set_read_timeout(Some(std::time::Duration::new(2, 0)))
        .map_err(SomeError::Io)?;
    sock.connect(server_sa).map_err(SomeError::Io)?;

    let time_now = Instant::now();
    let mut builder = Builder::new_query(1, true);
    builder.add_question(&prm.hostname, false, QueryType::A, QueryClass::IN);
    let packet = builder.build().unwrap_or_else(|x| x);

    sock.send(&packet).map_err(SomeError::Io)?;
    let mut buf = vec![0u8; 4096];
    let recv_len = sock.recv(&mut buf).map_err(SomeError::Io)?;
    let pkt = Packet::parse(&buf).map_err(SomeError::DnsParser)?;

    if pkt.header.response_code != ResponseCode::NoError
        && pkt.header.response_code != ResponseCode::NameError
    {
        return Err(SomeError::Other("Something bad happening"));
    }

    Ok(Ans {
        time: time_now.elapsed().subsec_micros(),
        len: recv_len,
    })
}
