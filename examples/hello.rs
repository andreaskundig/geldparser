use anyhow::{anyhow, Result};
use chrono::NaiveDate;
use geldparser::odf_transactions::old_booked_payments;

fn main() -> Result<()> {
    let start_date = NaiveDate::from_ymd(2017, 12, 31);
    let map = old_booked_payments(&start_date)?;
    let some_date = map.keys().last(); //.ok_or(anyhow!("no last date"))?;
    let query_date = some_date.ok_or(anyhow!("no date"))?;
    let rows = map.get(query_date).ok_or(anyhow!("no rows"))?;
    let row = rows.get(0).ok_or(anyhow!("no row"))?;
    let (amount, desc) = row;
    println!("{:?} {:?} {:?}", query_date, amount, desc);
    Ok(())
}
