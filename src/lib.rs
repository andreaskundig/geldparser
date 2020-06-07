#[macro_use]
extern crate lazy_static;
extern crate anyhow;
extern crate chrono;
extern crate failure;
extern crate regex;
extern crate rust_decimal;
use crate::accounts::choose_account_from_command_line;
use crate::accounts::{extract_recipient, Recipient, Account::*,  Eequity::*};
use chrono::NaiveDate;
use mt940::Message;
use mt940::{parse_mt940, sanitizers, StatementLine};
use regex::Regex;
use rust_decimal::Decimal;
use std::{borrow::Cow, fmt, fs, fs::File, io::prelude::*};
pub mod accounts;
pub mod files;
use crate::files::ebanking_payments;
use anyhow::Result;
use failure::Fail;
use itertools::{structs::GroupBy, Itertools};

// pub fn run(config: Config){
pub fn run(config: Config) -> Result<()> {
    let _date_to_payment = ebanking_payments()?;
    let input_filename = &config.input_filename;
    let mut output_file = File::create(&config.output_filename)?;
    println!("; {} -> {}", input_filename, &config.output_filename);
    writeln!(output_file, "; {:?}\n", input_filename)?;
    let messages = parse_messages(&input_filename)?;

    let start_date = NaiveDate::from_ymd(2019, 01, 01);

    if let Some(opening_message) = first_after(&start_date, &messages) {
        writeln!(
            output_file,
            "{}\n",
            Transaction::new_opening_balance(opening_message)
        )?;
    }

    let grouped = &stmtlines_after_grouped(&start_date, &messages);
    for (_date, stmtlines_group) in grouped {
        //TODO find ebanking lines, make detailed transactions
        for stmtline in stmtlines_group {
            write_stmtline(&mut output_file, stmtline, &config)?;
        }
    }
    Ok(())
}

pub fn parse_messages(input_filename: &str) -> Result<Vec<Message>> {
    let contents = fs::read_to_string(input_filename)?;
    let sanitized = sanitizers::sanitize(&contents[..]);
    parse_mt940(&sanitized[..]).map_err(|e| From::from(e.compat()))
}

pub fn first_after<'a>(
    &opening_balance_date: &NaiveDate,
    messages: &'a [Message],
) -> Option<&'a Message> {
    messages
        .iter()
        .find(|&m| m.opening_balance.date >= opening_balance_date)
}

// fn get_stmtline_value_date(stmtline: && StatementLine) -> NaiveDate{
//     stmtline.value_date
// }

pub fn stmtlines_after_grouped<'a>(
    start_date: &'a NaiveDate,
    messages: &'a [Message],
) -> GroupBy<
    &'a NaiveDate,
    impl Iterator<Item = &'a StatementLine>,
    // A closure that does not move, borrow,
    // or otherwise access (capture) local variables
    // is coercable to a function pointer (fn).
    fn(&&'a StatementLine) -> &'a NaiveDate, // The other solution would be to box the closure
                                             // like this: .group_by(Box::new(|s| &s.value_date))
                                             // and declare this type instead of fn:
                                             // Box<dyn Fn(& &'a StatementLine) -> &'a NaiveDate>
> {
    stmtlines_after(start_date, messages).group_by(|s| &s.value_date)
    // .group_by(get_stmtline_value_date)
}

pub fn stmtlines_after<'a>(
    start_date: &'a NaiveDate,
    messages: &'a [Message],
) -> impl Iterator<Item = &'a StatementLine> {
    messages
        .iter()
        .flat_map(|m| &m.statement_lines)
        .filter(move |s| &s.value_date >= start_date)
}

fn write_stmtline(
    output_file: &mut File,
    statement: &StatementLine,
    config: &Config,
) -> std::io::Result<()> {
    let owner_info = extract_info_to_owner(statement).unwrap_or("");
    let mut recipient = extract_recipient(owner_info);
    if config.interactive {
        let init_acc = recipient.account;
        let account_o = choose_account_from_command_line(init_acc, owner_info);
        let account = account_o.expect("Choosing error");
        recipient = Recipient {
            name: String::from(owner_info),
            account,
        };
        println!(";;{}\n", account);
    }
    let transaction = Transaction::new(statement, recipient);
    writeln!(output_file, "{}\n", transaction)?;
    Ok(())
}

struct Transaction<'a> {
    recipient: Recipient,
    details: Option<&'a str>,
    info_to_owner: Option<&'a str>,
    date: &'a NaiveDate,
    amount: &'a Decimal,
}

impl<'a> Transaction<'a> {
    fn new_opening_balance(message: &'a Message) -> Transaction<'a> {
        let recipient = Recipient {
            name: String::from("Checking Balance"),
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
    fn new(statement: &'a mt940::StatementLine, recipient: Recipient) -> Transaction<'a> {
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
            remove_newlines(&self.recipient.name),
            details_str,
            remove_newlines(info_to_owner_str),
            self.recipient.account,
            self.amount
        )
    }
}

fn extract_info_to_owner(statement: &mt940::StatementLine) -> Option<&str> {
    let oi = statement.information_to_account_owner.as_ref()?;
    Some(oi.as_str())
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
    pub input_filename: String,
    pub output_filename: String,
    pub interactive: bool,
}

impl Config {
    pub fn new(args: &[String]) -> Config {
        let mut input_filename = String::from("../bewegungen/2019.mt940");
        let output_filename = String::from("./output.ledger");
        let interactive = args.iter().find(|&arg| &arg == &"-i").is_some();
        let filename_index = if interactive { 2 } else { 1 };
        if args.len() > filename_index {
            input_filename = args[filename_index].clone();
        }
        Config {
            input_filename,
            interactive,
            output_filename,
        }
    }
}
