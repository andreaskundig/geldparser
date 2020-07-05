extern crate csv;

// Import the standard library's I/O module so we can read from stdin.
use std::error::Error;

use geldparser::csv_orders::ebanking_payments;

// The `main` function is where your program starts executing.
fn main() -> Result<(), Box<dyn Error>> {
    let date_to_payment = ebanking_payments()?;
    println!("Map: {:?}", date_to_payment);
    Ok(())
}
