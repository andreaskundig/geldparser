use chrono::NaiveDate;
use geldparser::{first_after, parse_messages, stmtlines_after, stmtlines_after_grouped};

static DT: fn(i32, u32, u32) -> chrono::NaiveDate = NaiveDate::from_ymd;

#[test]
fn it_finds_first_message() {
    let messages = parse_messages("./tests/demo.mt940").unwrap();
    assert_eq!(29, messages.len());

    let op = first_after(&DT(2019, 01, 01), &messages);
    assert_eq!(op, None);

    let first = first_after(&DT(2009, 01, 02), &messages).unwrap();
    assert_eq!(first.opening_balance.date, DT(2009, 01, 04));
}

#[test]
fn it_iterates_statement_lines() {
    let messages = parse_messages("./tests/demo.mt940").unwrap();
    assert_eq!(29, messages.len());

    let date0 = DT(2019, 01, 01);
    let lines0: Vec<_> = stmtlines_after(&date0, &messages).collect();
    assert_eq!(lines0.len(), 0);

    let date1 = DT(2009, 01, 01);
    let lines1: Vec<_> = stmtlines_after(&date1, &messages).collect();
    assert_eq!(lines1.len(), 45);
}

#[test]
fn it_groups_statement_lines() {
    let messages = parse_messages("./tests/demo.mt940").unwrap();
    assert_eq!(29, messages.len());

    let date1 = DT(2009, 01, 01);
    let mut stmt_count = 0;
    let mut group_count = 0;
    for (_date, group) in &stmtlines_after_grouped(&date1, &messages) {
        let stmtlines: Vec<_> = group.collect();
        //println!("lines {} {}", date, stmtlines.len());
        stmt_count += stmtlines.len();
        group_count += 1;
    }
    assert_eq!(stmt_count, 45);
    assert_eq!(group_count, 33);
}
