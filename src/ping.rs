extern crate clap;
extern crate dns_parser;

use clap::{value_t, ArgMatches};
use dns_parser::{Builder, Packet, ResponseCode};
use dns_parser::{Class, QueryClass, QueryType, RData};
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
            SomeError::Io(ref err) => write!(f, "IO: {}", err),
            SomeError::DnsParser(ref err) => write!(f, "DNSParser: {:?}", err),
            SomeError::Other(ref err) => write!(f, "Other: {}", err),
        }
    }
}
#[derive(Debug, Copy, Clone)]
struct Opt<'a> {
    hostname: &'a str,
    server: &'a str,
    count: u32,
    interval: u64,
    port: u32,
    query_type: QueryType,
    verbose: bool,
    ipv4: bool,
    ipv6: bool
}

fn parse_qtype(v: &str) -> QueryType {
    match v {
        "A" => QueryType::A,
        "AAAA" => QueryType::AAAA,
        "CNAME" => QueryType::CNAME,
        "MX" => QueryType::MX,
        "NS" => QueryType::NS,
        "PTR" => QueryType::PTR,
        "SOA" => QueryType::SOA,
        "SRV" => QueryType::SRV,
        "TXT" => QueryType::TXT,
        "All" => QueryType::All,
        _ => {
            println!("Error: wrong/not implemented qtype");
            std::process::exit(1);
        }
    }
}
pub fn dnsping(args: &ArgMatches) {
    let prm = Opt {
        server: &value_t!(args, "server", String).unwrap(),
        hostname: &value_t!(args, "hostname", String).unwrap(),
        count: value_t!(args, "count", u32).unwrap(),
        interval: value_t!(args, "interval", u64).unwrap(),
        port: value_t!(args, "port", u32).unwrap(),
        ipv4: args.is_present("ipv4"),
        ipv6: args.is_present("ipv6"),
        query_type: parse_qtype(&value_t!(args, "qtype", String).unwrap()),
        //query_type: value_t!(args, "qtype", QueryType),
        verbose: args.is_present("verbose"),
    };
    dbg!(prm);

    println!(
        "dnsdg ping server: {}, hostname: {}",
        prm.server, prm.hostname
    );
    let results: Vec<u32> = (0..prm.count)
        .map(|c| match do_it(prm) {
            Ok((time, len)) => {
                println!(
                    "{} bytes from {}: seq={:<3} time={:.3} ms ",
                    len,
                    prm.server,
                    c,
                    time as f32 / 1000.0
                );
                std::thread::sleep(std::time::Duration::from_millis(prm.interval));
                time
            }
            Err(e) => {
                println!("Error: {:?}", e);
                std::process::exit(1);
            }
        })
        .collect();
    //dbg!(results); moved?
    let max = results.iter().max().unwrap().clone() as f32;
    let min = results.iter().min().unwrap().clone() as f32;
    let sum: u32 = results.iter().sum();
    let aver: f32 = sum as f32 / results.len() as f32;
    println!(
        "min={} ms, max={} ms, avg={} ms",
        min / 1000.0,
        max / 1000.0,
        aver / 1000.0
    );
}

fn prs2(name: &str, port: u32) -> Result<SocketAddr, SomeError> {
    match name.to_socket_addrs() {
        Err(_) => match format!("{}:{}", name, port).to_socket_addrs() {
            Err(e) => Err(SomeError::Io(e)),
            Ok(o) => o
                .clone()
                .next()
                .ok_or_else(|| SomeError::Other("SockAddr+port.error")),
        },
        Ok(o) => o
            .clone()
            .next()
            .ok_or_else(|| SomeError::Other("SockAddr.error")),
    }
}

fn do_it(prm: Opt) -> Result<(u32, usize), SomeError> {
    let server_sa = prs2(prm.server, prm.port)?;
    //dbg!(server_sa); // new macro in 1.32
    //let sock = (match server_sa.is_ipv6() {
    //    true => UdpSocket::bind("[::]:0"),
    //    _ => UdpSocket::bind("0.0.0.0:0"),
    //}).map_err(SomeError::Io)?;
    
    let sock = (match prm.ipv4 {
        true => UdpSocket::bind("0.0.0.0:0"),
        _ => match prm.ipv6{
            true => UdpSocket::bind("[::]:0"),
            _ => match server_sa.is_ipv6() {
                true => UdpSocket::bind("[::]:0"),
                _ => UdpSocket::bind("0.0.0.0:0"),
            }
                
        }
    }).map_err(SomeError::Io)?;
    //println!("sock: {:?}", sock);

    sock.set_read_timeout(Some(std::time::Duration::new(2, 0)))
        .map_err(SomeError::Io)?;
    sock.connect(server_sa).map_err(SomeError::Io)?;

    let time_now = Instant::now();
    let mut builder = Builder::new_query(1, true);
    builder.add_question(&prm.hostname, false, prm.query_type, QueryClass::IN);
    let packet = builder.build().unwrap_or_else(|x| x);

    sock.send(&packet).map_err(SomeError::Io)?;
    let mut buf = vec![0u8; 4096];
    let recv_len = sock.recv(&mut buf).map_err(SomeError::Io)?;

    //let pkt = Packet::parse(&buf).map_err(SomeError::DnsParser)?;
    let pkt = match Packet::parse(&buf) {
        Ok(o) => o,
        Err(e) => match e {
            dns_parser::Error::InvalidType(_) => {
                return Ok((time_now.elapsed().subsec_micros(), recv_len))
            }
            _ => return Err(SomeError::DnsParser(e)),
        },
    };

    if pkt.header.response_code != ResponseCode::NoError
        && pkt.header.response_code != ResponseCode::NameError
    {
        return Err(SomeError::Other("Something bad happening"));
    }
    if prm.verbose {
        println!("got {} answers:", pkt.answers.len());
        for a in pkt.answers {
            println!(
                "    {} {} {} {}",
                a.name,
                a.ttl,
                match a.cls {
                    Class::IN => "IN",
                    Class::CS => "CS",
                    Class::CH => "CH",
                    Class::HS => "HS",
                },
                match a.data {
                    RData::A(dns_parser::rdata::a::Record(d)) => format!("A {}", d),
                    RData::AAAA(dns_parser::rdata::aaaa::Record(d)) => format!("AAAA {}", d),
                    RData::CNAME(dns_parser::rdata::cname::Record(d)) => format!("CNAME {}", d),
                    #[cfg_attr(rustfmt, rustfmt::skip)]
                    RData::MX(dns_parser::rdata::mx::Record{preference, exchange})
                        => format!("MX {} {}", preference, exchange),
                    RData::NS(dns_parser::rdata::ns::Record(d)) => format!("NS {}", d),
                    RData::PTR(dns_parser::rdata::ptr::Record(d)) => format!("PTR {}", d),
                    #[cfg_attr(rustfmt, rustfmt::skip)]
                    RData::SOA(dns_parser::rdata::soa::Record {
                        primary_ns, mailbox, serial, refresh, retry, expire, minimum_ttl,
                    }) => format!(
                        "SOA {} {} {} {} {} {} {}",
                        primary_ns, mailbox, serial, refresh, retry, expire, minimum_ttl
                    ),
                    #[cfg_attr(rustfmt, rustfmt::skip)]
                    RData::SRV(dns_parser::rdata::srv::Record{priority, weight, port, target})
                        => format!("SRV {} {} {} {}", priority, weight, port, target),
                    #[cfg_attr(rustfmt, rustfmt::skip)]
                    RData::TXT(ref txt) => {
                        let s = txt.iter().map(|x|
                           std::str::from_utf8(x).unwrap()).collect::<Vec<_>>().concat();
                        format!("TXT {}", s)
                    }
                    RData::Unknown(d) => format!("Unknown {:?}", &d),
                }
            );
        }
        let mut flags: std::vec::Vec<String> = std::vec::Vec::new();
        if !pkt.header.query {
            flags.push("QR".to_string());
        }
        if pkt.header.authoritative {
            flags.push("AA".to_string());
        }
        if pkt.header.truncated {
            flags.push("TC".to_string());
        }
        if pkt.header.recursion_desired {
            flags.push("RD".to_string());
        }
        if pkt.header.recursion_available {
            flags.push("RA".to_string());
        }
        if pkt.header.authenticated_data {
            flags.push("AD".to_string());
        }
        if pkt.header.checking_disabled {
            flags.push("CD".to_string());
        }
        println!("flags: {}", flags.join(" "));
    }

    Ok((time_now.elapsed().subsec_micros(), recv_len))
}
