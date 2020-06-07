//! Demonstrates how to block read events.
//!
//! cargo run --example event-read

use geldparser::accounts::{choose_account_from_command_line, Account::*, Eapartment::*, Eexpenses::*};

fn main() {
    let account = Expenses(Apartment(Electricity));
    match choose_account_from_command_line(account, "hi") {
        Err(why) => println!("Error: {}", why),
        Ok(chosen) => println!("You chose: {}", chosen),
    };
}
