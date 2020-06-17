extern crate anyhow;
extern crate csv;

use anyhow::Result;
use chrono::NaiveDate;
use csv::{ReaderBuilder, StringRecord};
use std::fs;
use std::fs::File;
use std::io;
use std::{collections::HashMap, ffi::OsString};
use itertools::Itertools;

pub fn ebanking_payments() -> Result<HashMap<NaiveDate, Vec<StringRecord>>> {
    let paths = fs::read_dir("../bewegungen/pain")?
        .map(|res| res.map(|e| e.path().into_os_string()))
        .filter(|n| match n {
            Ok(filename) => filename.to_string_lossy().contains("Aufträge "),
            _ => false,
        })
        .collect::<Result<Vec<_>, io::Error>>()?;
    build_map(&paths)
}

fn build_map(paths: &Vec<OsString>) -> Result<HashMap<NaiveDate, Vec<StringRecord>>> {

    let files = paths.iter().map(File::open).collect::<Result<Vec<_>,_>>()?;
    let mut readers: Vec<_>= files.iter()
        .map(|f| ReaderBuilder::new().delimiter(b';').from_reader(f))
        .collect();
    let records = readers.iter_mut()
        .map(|rdr| rdr.records().collect::<Vec<_>>())
        .filter(|records| records.len() > 1)
        .flatten()
        .collect::<Result<Vec<StringRecord>,_>>()?;
    let map_entries = records.into_iter()
        .map(|record|{
            let date_string = &record[0];
            match NaiveDate::parse_from_str(date_string, "%d.%m.%Y"){
                Ok(date) => Ok((date, record)),
                Err(e) => return Err(e),
            }
        })
        .collect::<Result<Vec<(NaiveDate, StringRecord)>,_>>()?;
      Ok(map_entries.into_iter().into_group_map())
}
