extern crate csv;

// Import the standard library's I/O module so we can read from stdin.
use chrono::NaiveDate;
use csv::ReaderBuilder;
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs::File;
use std::process;

use geldparser::files::ebanking_payments;

// The `main` function is where your program starts executing.
fn main() -> Result<(), Box<dyn Error>> {
    let date_to_payment = ebanking_payments()?;
    println!("Map: {:?}", date_to_payment);
    Ok(())
}
