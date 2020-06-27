use chrono::NaiveDate;
use geldparser::{first_after, parse_messages, stmtlines_after, stmtlines_grouped_by_date_after};
use mt940::parse_mt940;

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
    for (_date, group) in &stmtlines_grouped_by_date_after(&date1, &messages) {
        let stmtlines: Vec<_> = group.collect();
        //println!("lines {} {}", date, stmtlines.len());
        stmt_count += stmtlines.len();
        group_count += 1;
    }
    assert_eq!(stmt_count, 45);
    assert_eq!(group_count, 33);
}

#[test]
fn it_parses(){
    let input = "\
    :20:E00000000A97C5DA
:25:CH6700700111800017056
:28C:1/1
:60F:C190103CHF269271,61
:61:1812280103D18,NMSCNONREF//IBEP010300000893
Einkauf ZKB Maestro Karte
:86:?ZKB:2218 Einkauf ZKB Maestro Karte Nr. 73817865, CAFE GLACIER
REMOR 1204 GENEVE
:61:1812310103D7,2NMSCNONREF//IBEP010300187754
Einkauf ZKB Maestro Karte
:86:?ZKB:2218 Einkauf ZKB Maestro Karte Nr. 73817865, Philippe Taille
1207 Geneve
:61:1812290103D142,95NMSCNONREF//IBEI010300002498
Einkauf ZKB Maestro Karte
:86:?ZKB:2218
Einkauf ZKB Maestro Karte Nr. 73817865, GRAND FRAIS 01210 FERNEY
:61:1812290103D36,31NMSCNONREF//IBEI010300002541
Einkauf ZKB Maestro Karte
:86:?ZKB:2218 Einkauf ZKB Maestro Karte Nr. 73817865, Centre Leclerc
(Ferney 1210
:61:190103D858,7NMSCNONREF//Z190039310263
eBill  (2)
:86:?ZKB:2214
Gemaess Ihrem eBanking Auftrag
?ZI:?9:2
:61:190103D1612,95NMSCBLVL-1-190101215//Z190039310195
eBanking (4)
:86:?ZKB:2214
Gemaess Ihrem eBanking Auftrag BLVL-1-19010121525544
?ZI:?9:4
:61:190103D232,24NTRFBLVL-2-190101215//Z190039310187
eBanking: IVI Madrid, Avenida del
:86:?ZKB:2214 IVI Madrid
Avenida del Talgo, 68
ES-28023 Madrid
maintenance Andreas Kundig
Gemaess Ihrem eBanking Auftrag BLVL-2-19010121525545
?ZI:?3:1,132886?9:1
:62F:C190103CHF266363,26
:64:C190103CHF266363,26
";

    let input_parsed = parse_mt940(input).unwrap();
    assert_eq!(1, input_parsed.len());
    let transaction = &input_parsed[0];
    println!("transa {:?}", transaction);
    assert_eq!(7, transaction.statement_lines.len());
    let line6 = &transaction.statement_lines[6];
    assert!(line6.supplementary_details.is_some());
    assert_eq!("eBanking: IVI Madrid, Avenida del",
               line6.supplementary_details.as_ref().unwrap());
    assert_eq!(input_parsed[0].transaction_ref_no, "E00000000A97C5DA");
}
