#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate chrono;

use mt940::parse_mt940;
use mt940::sanitizers;
use regex::Regex;
use std::borrow::Cow;
use std::fs;
use chrono::NaiveDate;
use text_io::read;

// fn accounts() -> [std::string::String;3]{
//     return [String::from("Expenses::Maestro"),
//             String::from("Expenses::Appartment::Electricity"),
//             String::from("Expenses::Rest")];

// }

// fn accounts() -> std::string::String{
//     return String::from("Expenses::Maestro")
// }

pub fn run(config: Config) {
    let filename = config.filename;
    println!("{:?}", filename);

    // let mut input = String::new();
    // match io::stdin().read_line(&mut input) {
    //     Ok(n) => {
    //         println!("{} bytes read", n);
    //         println!("{}", input);
    //     }
    //     Err(error) => println!("error: {}", error),
    // }
    
    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");

    let sanitized = sanitizers::sanitize(&contents[..]);
    let messages = parse_mt940(&sanitized[..]).unwrap();

    let start_date = NaiveDate::from_ymd(2019, 01, 01);

    println!("messages {:?}", messages.len());
    for message in &messages {
        println!("=======================================");
        println!("opening balance {:?} lines {:?}",
                 message.opening_balance.amount, message.statement_lines.len());
        println!("=======================================");
        //let statement = &message.statement_lines[0];
        for statement in &message.statement_lines {
            
            let date = statement.entry_date.unwrap_or(statement.value_date);
            if date < start_date {
                continue;
            }
            println!(
                "{} {}\n {:?}",
                date,
                statement.supplementary_details.as_ref().unwrap(),
                statement.amount,
            );
            let entry_o = extract_transaction(statement);
            if entry_o.is_some() {
                let entry = entry_o.unwrap();
                println!("recipient: {}", remove_newlines(entry.recipient));
                println!("account: {}", entry.account);
            }
            let owner_info = statement
                .information_to_account_owner
                .as_ref()
                .map(String::as_ref)
                .unwrap_or("no info");
            println!("owner info: {}", owner_info);
            println!("------------------------");

            loop {
                let input: String = read!("{}\n");
                println!("next...{}", input);
                if input.len() == 0 { break; }
            }
        }
    }
}

struct Transaction<'a> {
    recipient: &'a str,
    account: &'a str,
}

fn extract_transaction(statement: &mt940::StatementLine) -> Option<Transaction> {
    let owner_info = extract_owner_info(statement);
    let _extractors: Vec<fn(&str) -> Option<Transaction>> = vec![
        extract_maestro_transaction,
        extract_sig_transaction,
        extract_rest_transaction,
    ];

    owner_info.and_then({ |oi| _extractors.iter().find_map({ |f| f(oi) }) })
}

fn extract_owner_info(statement: &mt940::StatementLine) -> Option<&str> {
    let op_string_ref: Option<&String> = statement.information_to_account_owner.as_ref();
    // https://stackoverflow.com/questions/31233938/converting-from-optionstring-to-optionstr
    op_string_ref.map(String::as_str)
}

fn extract_maestro_transaction(owner_info: &str) -> Option<Transaction> {
    // https://rust-lang-nursery.github.io/rust-cookbook/text/regex.html
    lazy_static! {
        static ref MAESTRO: Regex =
            Regex::new(r"(?s).*Einkauf ZKB Maestro Karte Nr. 73817865[^,]*,(.*$)").unwrap();
    }
    let c = MAESTRO.captures(owner_info);
    c.and_then(|cap| {
        cap.get(1).map(|m| Transaction {
            recipient: m.as_str(),
            account: "Expenses::Maestro",
        })
    })
}

fn extract_sig_transaction(owner_info: &str) -> Option<Transaction> {
    // https://rust-lang-nursery.github.io/rust-cookbook/text/regex.html
    lazy_static! {
        static ref SIG: Regex = Regex::new(r"Services Industriels de Geneve").unwrap();
    }

    if SIG.is_match(owner_info) {
        Some(Transaction {
            recipient: "Services Industriels de Geneve",
            account: "Expenses::Appartment::Electricity",
        })
    } else {
        None
    }
}

fn extract_rest_transaction(owner_info: &str) -> Option<Transaction> {
    Some(Transaction {
        recipient: owner_info,
        account: "Expenses::Rest",
    })
}

//TODO understand cow
fn remove_newlines(text: &str) -> Cow<str> {
    // https://rust-lang-nursery.github.io/rust-cookbook/text/regex.html
    lazy_static! {
        static ref NEWLINE: Regex = Regex::new(r"\n").unwrap();
    }
    NEWLINE.replace_all(text, "; ")
}

pub struct Config {
    pub filename: String,
}

impl Config {
    pub fn new(args: &[String]) -> Config {
        let mut filename = String::from("../bewegungen/2019.mt940");
        if args.len() > 1 {
            filename = args[1].clone();
        }
        Config { filename }
    }
}
