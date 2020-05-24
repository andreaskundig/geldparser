extern crate csv;

// Import the standard library's I/O module so we can read from stdin.
use chrono::NaiveDate;
use csv::ReaderBuilder;
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs::File;
use std::process;

use geldparser::files::print_files;

// The `main` function is where your program starts executing.
fn main() -> Result<(), Box<dyn Error>> {
    print_files()?;
    if let Err(err) = run() {
        println!("{}", err);
        process::exit(1);
    }
    Ok(())
}

fn get_first_arg() -> Result<OsString, Box<dyn Error>> {
    match env::args_os().nth(1) {
        //None => Err(From::from("expected 1 argument, but got none")),
        None => Ok("../bewegungen/pain/Aufträge 20200102182712.csv".into()),
        Some(file_path) => Ok(file_path),
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let file_path = get_first_arg()?;
    let file = File::open(file_path)?;
    let mut rdr = ReaderBuilder::new().delimiter(b';').from_reader(file);
    for result in rdr.records() {
        let record = result?;
        let date_string = &record[0];
        let date = NaiveDate::parse_from_str(date_string, "%d.%m.%Y")?;
        println!("{:?} {:?}", date, record);
    }
    Ok(())
}
