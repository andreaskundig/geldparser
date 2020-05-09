extern crate geldparser;
use geldparser::accounts::ACCOUNTS;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("hello example {}", ACCOUNTS[1]);
    println!("hello example debug {:?}", ACCOUNTS[1]);
    println!("args {:?}", args);
    let interactive = args.iter().find(|&arg| &arg == &"-i").is_some();
    println!("interactive {:?}", interactive);
}
