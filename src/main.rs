extern crate clap;
extern crate dns_parser;

mod ping;
use clap::{App, Arg, SubCommand};

fn main() {
    //let args: Vec<String> = env::args().collect();

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
                ).arg(
                    Arg::with_name("server")
                        .help("DNS Server")
                        .short("d")
                        .long("dnsserver")
                        .takes_value(true)
                        .value_name("server")
                        .default_value("8.8.8.8"),
                ).arg(
                    Arg::with_name("interval")
                        .help("Time between each request, in seconds")
                        .short("i")
                        .long("interval")
                        .takes_value(true)
                        .value_name("interval")
                        .default_value("1"),
                ).arg(
                    Arg::with_name("count")
                        .help("Count times")
                        .short("c")
                        .long("count")
                        .takes_value(true)
                        .value_name("count")
                        .default_value("10"),
                ),
        ).subcommand(
            SubCommand::with_name("some").arg(
                Arg::with_name("some")
                    .help("some other command")
                    .short("s")
                    .long("some"),
            ),
        );
    let matches = app.get_matches();
    //println!("found: {:?}", matches);

    if let ("ping", Some(cmd)) = matches.subcommand() {
        ping::dnsping(&cmd);
    }else{
        println!("run as dnsdg ping");
    }
}

fn _print_usage(err: &str) {
    println!("Err: {}", err);
    println!("Usage: bla-bla-bla");
}
