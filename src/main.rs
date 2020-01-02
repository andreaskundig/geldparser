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
    let input_parsed = parse_mt940(&sanitized[..]).unwrap();
    println!("{:?}", input_parsed);
}

struct Config {
    filename: String,
}

impl Config {
    fn new(args: &[String]) -> Config {
        if args.len() < 2 {
            panic!("not enough arguments");
        }
        let filename = args[1].clone();

        Config { filename }
    }
}
