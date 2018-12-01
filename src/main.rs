use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.get(1) {
        Some(x) => match &x[..] {
            "ping" => dnsping(&args),
            _ => print_usage("wrong cmd"),
        },
        None => print_usage("cmd not found"),
    }
}

fn print_usage(err: &str) {
    println!("Err: {}", err);
    println!("Usage: bla-bla-bla");
}

#[derive(Debug)]
struct PingOpts <'a> {
    a: &'a str,
}

impl Default for PingOpts {
    fn default() -> PingOpts {
        PingOpts {
            a: "String1";
        }
    }
}

fn dnsping(args: &Vec<String>) {

    let prm = PingOpts {
        a: "SomeOne".to_owned(),
        ..Default::default()
    };
    println!("{:?}", prm);
    println!("{:?}", args);
}
