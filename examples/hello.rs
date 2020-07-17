use anyhow::{anyhow, Result};
use calamine::DataType;
use chrono::NaiveDate;
use geldparser::odf_transactions::{
    build_map_after, extract_date, open_worksheet_range,
};

fn print_row(row: &[DataType]) -> Result<()> {
    let date = extract_date(row).ok_or(anyhow!("no date"))?;
    let amount = row[4].get_float().ok_or(anyhow!("no amount"))?;
    let desc = row[7].get_string().unwrap_or("");
    println!("{:?} {:?} {:?}", date, amount, desc);
    Ok(())
}

// fn old_booked_payments(start_date: &NaiveDate) -> Result<HashMap<NaiveDate, Vec<&[DataType]>>> {
//     let path = "../Geld.ods";
//     let range = open_worksheet_range(path)?;
//     build_map_after(start_date, &range)
// }

fn main() -> Result<()> {
    let start_date = NaiveDate::from_ymd(2017, 12, 31);
    let path = "../Geld.ods";
    let range = open_worksheet_range(path)?;
    let mut last_date = None;
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
            print_row(row)?;
            // let amount = row[4].get_float().ok_or(anyhow!("no amount"))?;
            // let desc = row[7].get_string().unwrap_or("");
            // println!("{:?} {:?} {:?}", date, amount, desc);
            last_date = Some(date);
            Ok(())
        })
        .collect::<Result<()>>()?;

    let range = open_worksheet_range(path)?;
    let map = build_map_after(&start_date, &range)?;
    let some_date = map.keys().last(); //.ok_or(anyhow!("no last date"))?;
    let query_date =
        last_date.as_ref().or(some_date).ok_or(anyhow!("no last date"))?;
    let rows = map.get(query_date).ok_or(anyhow!("no rows"))?;
    let row = rows.get(0).ok_or(anyhow!("no row"))?;
    print_row(row)
}
