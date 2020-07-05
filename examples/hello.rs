use anyhow::{anyhow, Result};
use calamine::{open_workbook, DataType, Ods, Reader};
use chrono::NaiveDate;

fn extract_date(row: &[DataType]) -> Option<NaiveDate> {
    row[1]
        .get_string()
        .map(|date_str| {
            NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()
        })
        .flatten()
}

fn main() -> Result<()> {
    let path = "../Geld.ods"; // "../Geld-old.xlsx";
    let mut workbook: Ods<_> =
        open_workbook(path).expect("Cannot open file");
    let names = workbook.sheet_names();
    println!("sheets {:?}", names);

    let start_date = NaiveDate::from_ymd(2017, 12, 31);
    let range = workbook
        .worksheet_range("Ausgabe")
        .ok_or(anyhow!("Cannot find 'Ausgabe'"))??;
    range
        .rows()
        .skip(1)
        .filter(|row| {
            let date_o = extract_date(row);
            date_o.map(|date| date >= start_date).unwrap_or(false)
        })
        .take(10)
        .map(|row| -> Result<()> {
            let date = extract_date(row).ok_or(anyhow!("no date"))?;
            let amount = row[4].get_float().ok_or(anyhow!("no amount"))?;
            let desc = row[7].get_string().unwrap_or("");
            println!("{:?} {:?} {:?}", date, amount, desc);
            Ok(())
        }).collect()
}
