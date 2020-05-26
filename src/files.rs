extern crate csv;

use chrono::NaiveDate;
use csv::ReaderBuilder;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io;
use std::{collections::HashMap, ffi::OsString};

pub fn print_files() -> Result<(), Box<dyn Error>> {
    let paths = fs::read_dir("../bewegungen/pain")?
        .map(|res| res.map(|e| e.path().into_os_string()))
        .filter(|n| match n {
            Ok(filename) => filename.to_string_lossy().contains("Aufträge "),
            _ => false,
        })
        .collect::<Result<Vec<_>, io::Error>>()?;

    for path in &paths {
        println!("Name: {:?}", path)
    }
    build_map(&paths)?;
    Ok(())
}

fn build_map(paths: &Vec<OsString>) -> Result<(), Box<dyn Error>> {
    // let mut date_to_payment = HashMap::new();

    for path in paths {
        let file = File::open(&path)?;
        let mut rdr = ReaderBuilder::new().delimiter(b';').from_reader(file);
        for result in rdr.records() {
            let record = result?;
            let date_string = &record[0];
            let date = NaiveDate::parse_from_str(date_string, "%d.%m.%Y")?;
            println!("{:?} {:?}", date, record);
        }
        println!("Name: {:?}", path);
    }
    Ok(())
}
