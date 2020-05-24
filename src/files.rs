extern crate csv;

use std::error::Error;
use std::fs;
use std::io;

pub fn print_files() -> Result<(), Box<dyn Error>> {
    let paths = fs::read_dir("../bewegungen/pain")?
        .map(|res| 
            res.map(|e| e.file_name())
        )
        .filter(|n| 
            match n {
                Ok(filename) => filename.to_string_lossy().starts_with("Aufträge "),
                _ => false
            }
        )
        .collect::<Result<Vec<_>, io::Error>>()?;

    for path in paths {
        println!("Name: {:?}", path)
    }
    Ok(())
}
