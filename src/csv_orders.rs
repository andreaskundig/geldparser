extern crate anyhow;
extern crate csv;

use anyhow::{anyhow, Result};
use chrono::NaiveDate;
use csv::{ReaderBuilder, StringRecord};
use itertools::Itertools;
use rust_decimal::Decimal;
use std::fs;
use std::fs::File;
use std::io;
use std::str::FromStr;
use std::{collections::HashMap, ffi::OsString};

#[derive(Debug)]
pub struct Order {
    pub date: NaiveDate,
    pub amount: Decimal,
    pub description: String,
    pub is_chf: bool,
}

impl Order {
    fn new(record: &StringRecord) -> Result<Order> {
        Ok(Order {
            date: record
                .get(0)
                .ok_or(anyhow!("missing date"))
                .map(|ds| NaiveDate::parse_from_str(ds, "%d.%m.%Y"))??,
            amount: record
                .get(5)
                .map(|a| Decimal::from_str(a).ok())
                .flatten()
                .unwrap_or(Decimal::new(0, 0)),
            description: String::from(record.get(1).unwrap_or("")),
            is_chf: record
                .get(4)
                .map(|currency| currency == "CHF")
                .unwrap_or(false),
        })
    }
}

pub fn ebanking_payments_from_csvs(
) -> Result<HashMap<NaiveDate, Vec<Order>>> {
    let paths = fs::read_dir("../bewegungen/pain")?
        .map(|res| res.map(|e| e.path().into_os_string()))
        .filter(|n| match n {
            Ok(filename) => {
                filename.to_string_lossy().contains("Aufträge ")
            }
            _ => false,
        })
        .collect::<Result<Vec<_>, io::Error>>()?;
    build_map(&paths)
}

fn build_map(
    paths: &Vec<OsString>,
) -> Result<HashMap<NaiveDate, Vec<Order>>> {
    let files = paths
        .iter()
        .map(File::open)
        .collect::<Result<Vec<_>, _>>()?;
    let mut readers: Vec<_> = files
        .iter()
        .map(|f| ReaderBuilder::new().delimiter(b';').from_reader(f))
        .collect();
    let records = readers
        .iter_mut()
        .map(|rdr| rdr.records().collect::<Vec<_>>())
        .filter(|records| records.len() > 1)
        .flatten()
        .collect::<Result<Vec<StringRecord>, _>>()?;
    let map_entries = records
        .into_iter()
        .map(|record| {
            let order = Order::new(&record)?;
            Ok((order.date, order))
        })
        .collect::<Result<Vec<(NaiveDate, Order)>>>()?;
    Ok(map_entries.into_iter().into_group_map())
}
