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

pub fn build_map_after(date: NaiveDate, range: &Range<DataType>) -> Result<HashMap<NaiveDate, Vec<&[DataType]>>>{
    let map_entries = range
        .rows()
        .skip(1)
        .filter(|row| {
            let date_o = extract_date(row);
            date_o.map(|d| d >= date).unwrap_or(false)
        })
        .map(|row| -> Result<(NaiveDate, &[DataType])>{
            let date = extract_date(row).ok_or(anyhow!("no date"))?;
            Ok((date, row))
        })
        .collect::<Result<Vec<(NaiveDate, &[DataType])>>>()?;
    Ok(map_entries.into_iter().into_group_map())
}
// pub fn build_map(range: Range<DataType>) -> Result<HashMap<NaiveDate,Vec<>>>
