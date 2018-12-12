extern crate clap;
extern crate dns_parser;

mod ping;
use clap::{App, Arg, SubCommand};
//use dns_parser::{Class, QueryClass, QueryType, RData};

fn main() {
    let app = App::new("DNSPing")
        .subcommand(
            SubCommand::with_name("ping")
                .arg(
                    Arg::with_name("hostname")
                        .help("Hostname to resolve")
                        .short("n")
                        .long("name")
                        .takes_value(true)
                        .value_name("hostname")
                        .default_value("google.com"),
                )
                .arg(
                    Arg::with_name("verbose")
                        .help("Print actual server response")
                        .short("v")
                        .long("verbose"),
                )
                .arg(
                    Arg::with_name("qtype")
                        .help("DNS request record type")
                        .short("t")
                        .long("qtype")
                        .takes_value(true)
                        .value_name("qtype")
                        .default_value("A")
                        //.possible_values(&PrmQtype::variants()),
                        .possible_values(&[
"A", "AAAA", "CNAME", "MX", "NS", "PTR", "SOA", "SRV", "TXT", "All"
                    ])
                        //.possible_values(&QueryClass::variants()),
                )
                .arg(
                    Arg::with_name("server")
                        .help("DNS Server")
                        .short("d")
                        .long("dnsserver")
                        .takes_value(true)
                        .value_name("server")
                        .default_value("8.8.8.8"),
                )
                .arg(
                    Arg::with_name("port")
                        .help("DNS server port number")
                        .short("p")
                        .long("port")
                        .takes_value(true)
                        .value_name("port")
                        .default_value("53"),
                )
                .arg(
                    Arg::with_name("interval")
                        .help("Time between each request, in milliseconds")
                        .short("i")
                        .long("interval")
                        .takes_value(true)
                        .value_name("interval")
                        .default_value("1000"),
                )
                .arg(
                    Arg::with_name("count")
                        .help("Count times")
                        .short("c")
                        .long("count")
                        .takes_value(true)
                        .value_name("count")
                        .default_value("10"),
                ),
        )
        .subcommand(
            SubCommand::with_name("some").arg(
                Arg::with_name("some")
                    .help("some other command")
                    .short("s")
                    .long("some"),
            ),
        );
    let matches = app.get_matches();

    if let ("ping", Some(cmd)) = matches.subcommand() {
        ping::dnsping(&cmd);
    } else {
        println!("run as dnsdg ping");
    }
}
