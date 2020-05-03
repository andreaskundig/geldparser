extern crate geldparser;
use geldparser::accounts::ACCOUNTS;

fn main() {
    println!("hello example {}", ACCOUNTS[1]);
    println!("hello example debug {:?}", ACCOUNTS[1]);
}
