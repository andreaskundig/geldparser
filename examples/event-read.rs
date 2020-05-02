//! Demonstrates how to block read events.
//!
//! cargo run --example event-read

use geldparser::accounts::{choose_account_from_command_line};

fn main(){
    match choose_account_from_command_line(){
        Err(why) => println!("Error: {}", why),
        Ok(chosen) => println!("You chose: {}", chosen),
    };
}
