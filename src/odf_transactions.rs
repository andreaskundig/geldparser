use anyhow::{anyhow, Result, Error};
use calamine::{open_workbook, DataType, Ods, Range, Reader};
use chrono::NaiveDate;
use std::collections::HashMap;
use itertools::Itertools;

pub fn open_worksheet_range(path: &str) -> Result<Range<DataType>> {
    let mut workbook: Ods<_> = open_workbook(path)?;
    let names = workbook.sheet_names();
    println!("sheets {:?}", names);

    workbook
        .worksheet_range("Ausgabe")
        .ok_or(anyhow!("Cannot find 'Ausgabe'"))?
    // why is this not converted automatically to anyhow::Error?
        .map_err(Error::new)
}

pub fn extract_date(row: &[DataType]) -> Option<NaiveDate> {
    row[1]
        .get_string()
        .map(|date_str| {
            NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()
        })
        .flatten()
}

pub type RowTuple = (f64,String);
pub fn extract_tuple(row: &[DataType]) -> RowTuple{
    let desc = row[7].get_string().unwrap_or("");
    let amount_o: Option<f64> = row[4].get_float();
    if amount_o.is_none() {
        println!("Missing amount in row {:?}", row);
    }
    let amount = amount_o.unwrap_or(0.0);
    (amount, String::from(desc))
}

pub fn build_map_after<'a,'b>(date: &'a NaiveDate, range: &'b Range<DataType>) -> Result<HashMap<NaiveDate, Vec<RowTuple>>>{
    let map_entries = range
        .rows()
        .skip(1)
        .filter(|row| {
            let date_o = extract_date(row);
            date_o.map(|d| d >= *date).unwrap_or(false)
        })
        .map(|row| -> Result<(NaiveDate, RowTuple)>{
            let date = extract_date(row).ok_or(anyhow!("no date"))?;
            Ok((date, extract_tuple(row)))
        })
        .collect::<Result<Vec<(NaiveDate, RowTuple)>>>()?;
    Ok(map_entries.into_iter().into_group_map())
}

pub fn old_booked_payments(start_date: &NaiveDate) -> Result<HashMap<NaiveDate, Vec<RowTuple>>> {
    let path = "../Geld.ods";
    let range = open_worksheet_range(path)?;
    build_map_after(start_date, &range)
}
