use calamine::{Reader, open_workbook, Ods};
use chrono::NaiveDate;
use anyhow::{anyhow, Result};

fn main() -> Result<()>{
    let path = "../Geld.ods"; // "../Geld-old.xlsx";
    let mut workbook: Ods<_> = open_workbook(path).expect("Cannot open file");
    let names = workbook.sheet_names();
    println!("sheets {:?}", names);
    let range = workbook.worksheet_range("Ausgabe")
        .ok_or(anyhow!("Cannot find 'Ausgabe'"))??;
    for (i, row) in range.rows().skip(1).enumerate() {
        
        let date_str = row[1].get_string().ok_or(anyhow!("no date"))?;
        let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;
        let amount = row[4].get_float().ok_or(anyhow!("no amount"))?;
        let desc  = row[7].get_string().unwrap_or("");
        println!("{:?} {:?} {:?}", date , amount, desc);
        if i>10{
            break;
        }
    }
    Ok(())
}
