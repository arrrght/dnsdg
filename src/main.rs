extern crate dns_parser;

use std::env;
mod ping;

fn main() {
    let args: Vec<String> = env::args().collect();
    ping::dnsping(&args);
    //match args.get(1) {
    //    Some(x) => match &x[..] {
    //        "ping" => ping::dnsping(&args),
    //        _ => print_usage("wrong cmd"),
    //    },
    //    None => print_usage("cmd not found"),
    //}
}

fn _print_usage(err: &str) {
    println!("Err: {}", err);
    println!("Usage: bla-bla-bla");
}

