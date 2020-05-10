#[macro_use]
extern crate lazy_static;
extern crate chrono;
extern crate regex;
extern crate rust_decimal;
use crate::accounts::choose_account_from_command_line;
use crate::accounts::{Account::*, Apartment::*, Equity::*, Expenses::*, *};
use chrono::NaiveDate;
use mt940::{parse_mt940, sanitizers};
use regex::Regex;
use rust_decimal::Decimal;
use std::{borrow::Cow, fmt, fs};

pub mod accounts;

pub fn run(config: Config) {
    let filename = config.filename;
    println!("; {:?}", filename);

    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");

    let sanitized = sanitizers::sanitize(&contents[..]);
    let messages = parse_mt940(&sanitized[..]).unwrap();

    let start_date = NaiveDate::from_ymd(2019, 01, 01);

    for (index, message) in (&messages).iter().enumerate() {
        // for message in &messages {
        if index == 0 {
            println!("{}\n", Transaction::new_opening_balance(&message));
        }

        //let statement = &message.statement_lines[0];
        for statement in &message.statement_lines {
            let date = statement.entry_date.unwrap_or(statement.value_date);
            if date < start_date {
                continue;
            }
            let owner_info = extract_info_to_owner(statement).unwrap_or("");
            let mut recipient = extract_recipient(owner_info);
            if config.interactive {
                let init_acc = recipient.account;
                println!(";;{}\n", remove_newlines(owner_info));
                let account_o = choose_account_from_command_line(init_acc);
                let account = account_o.expect("Choosing error");
                recipient = Recipient{name: owner_info, account};
                println!(";;{}\n", account);
            }
            let transaction = Transaction::new(statement, recipient);
            println!("{}\n", transaction);
        }
    }
}

struct Transaction<'a> {
    recipient: Recipient<'a>,
    details: Option<&'a str>,
    info_to_owner: Option<&'a str>,
    date: &'a NaiveDate,
    amount: &'a Decimal,
}

struct Recipient<'a> {
    name: &'a str,
    account: Account,
}

impl<'a> Transaction<'a> {
    fn new_opening_balance(message: &'a mt940::Message) -> Transaction<'a> {
        let recipient = Recipient {
            name: "Checking Balance",
            account: Equity(OpeningBalances),
        };
        let info_to_owner = &message.information_to_account_owner;
        Transaction {
            date: &message.opening_balance.date,
            amount: &message.opening_balance.amount,
            recipient,
            details: Option::None,
            info_to_owner: info_to_owner.as_ref().map(String::as_str),
        }
    }
    fn new(statement: &'a mt940::StatementLine, recipient: Recipient<'a>) -> Transaction<'a> {
        let entry_date = statement.entry_date.as_ref();
        let date = entry_date.unwrap_or(&statement.value_date);
        let info_to_owner = extract_info_to_owner(statement);
        Transaction {
            date,
            recipient,
            info_to_owner,
            amount: &statement.amount,
            details: statement.supplementary_details.as_ref().map(String::as_str),
        }
    }
}

impl<'a> fmt::Display for Transaction<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let info_to_owner_str = self.info_to_owner.unwrap_or("-");
        let details_str = self.details.unwrap_or("-");
        //TODO only add a line for details/owner_info if they are present
        write!(
            f,
            "{} {}\n  ; {}\n  ; {}\n  {}             {:?}\n  Assets::Checking",
            self.date.format("%Y/%m/%d"),
            remove_newlines(self.recipient.name),
            details_str,
            remove_newlines(info_to_owner_str),
            self.recipient.account,
            self.amount
        )
    }
}

fn extract_recipient(owner_info: &str) -> Recipient {
    let _extractors: Vec<fn(&str) -> Option<Recipient>> = vec![
        extract_maestro_recipient,
        extract_sig_recipient,
    ];

    _extractors.iter().find_map({ |f| f(owner_info) })
        .unwrap_or_else({|| rest_recipient(owner_info)})
}

fn extract_info_to_owner(statement: &mt940::StatementLine) -> Option<&str> {
    let op_string_ref: Option<&String> = statement.information_to_account_owner.as_ref();
    // https://stackoverflow.com/questions/31233938/converting-from-optionstring-to-optionstr
    op_string_ref.map(String::as_str)
}

fn extract_maestro_recipient(owner_info: &str) -> Option<Recipient> {
    // https://rust-lang-nursery.github.io/rust-cookbook/text/regex.html
    lazy_static! {
        static ref MAESTRO: Regex =
            Regex::new(r"(?s).*Einkauf ZKB Maestro Karte Nr. 73817865[^,]*,(.*$)").unwrap();
    }
    let c = MAESTRO.captures(owner_info);
    c.and_then(|cap| {
        cap.get(1).map(|m| Recipient {
            name: m.as_str(),
            account: Expenses(Maestro),
        })
    })
}

fn extract_sig_recipient(owner_info: &str) -> Option<Recipient> {
    lazy_static! {
        static ref SIG: Regex = Regex::new(r"Services Industriels de Geneve").unwrap();
    }

    if SIG.is_match(owner_info) {
        Some(Recipient {
            name: "Services Industriels de Geneve",
            account: Expenses(Apartment(Electricity)),
        })
    } else {
        None
    }
}

fn rest_recipient(owner_info: &str) -> Recipient {
    Recipient {
        name: owner_info,
        account: Expenses(Rest),
    }
}

/* Cow
In Rust, the abbreviation “Cow” stands for “clone on write”.
 It is an enum with two states: Borrowed and Owned. This means you can use it to abstract over whether you own the data or just have a reference to it. This is especially useful when you want to return a type from a function that may or may not need to allocate.
https://deterministic.space/secret-life-of-cows.html
*/
fn remove_newlines(text: &str) -> Cow<str> {
    lazy_static! {
        static ref NEWLINE: Regex = Regex::new(r"\n").unwrap();
    }
    NEWLINE.replace_all(text, "; ")
}

pub struct Config {
    pub filename: String,
    pub interactive: bool,
}

impl Config {
    pub fn new(args: &[String]) -> Config {
        let mut filename = String::from("../bewegungen/2019.mt940");
        let interactive = args.iter().find(|&arg| &arg == &"-i").is_some();
        let filename_index = if interactive { 2 } else { 1 };
        if args.len() > filename_index {
            filename = args[filename_index].clone();
        }
        Config {
            filename,
            interactive,
        }
    }
}
