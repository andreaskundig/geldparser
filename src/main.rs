use mt940::parse_mt940;
use mt940::sanitizers;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args);
    let filename = config.filename;
    println!("{:?}", filename);

    let contents = fs::read_to_string(filename)
        .expect("Something went wrong reading the file");

    let sanitized = sanitizers::sanitize(&contents[..]);
    let messages = parse_mt940(&sanitized[..]).unwrap();
    let message = &messages[0];
    println!("{:?}", message.opening_balance.amount);
    println!("{:?}", message.statement_lines[0]);
}

struct Config {
    filename: String,
}

impl Config {
    fn new(args: &[String]) -> Config {
        let mut filename = String::from("../bewegungen/2019.mt940");
        if args.len() > 1 {
            filename = args[1].clone();
        }
        Config { filename }
    }
}
