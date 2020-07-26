#[macro_use]
extern crate lazy_static;
extern crate anyhow;
extern crate chrono;
extern crate failure;
extern crate regex;
extern crate rust_decimal;
use crate::accounts::choose_account_from_command_line;
use crate::accounts::{
    extract_recipient, Account::*, Eequity::*, Recipient,
    R_GROUPED_EBANKING, R_GROUPED_EBILL,
};
use chrono::NaiveDate;
use mt940::Message;
use mt940::{parse_mt940, sanitizers, StatementLine};
use regex::Regex;
use rust_decimal::{prelude::ToPrimitive, Decimal};
use std::{
    borrow::Cow, collections::HashMap, fmt, fs, fs::File, io::prelude::*,
};
pub mod accounts;
pub mod csv_orders;
pub mod odf_transactions;
use crate::csv_orders::{ebanking_payments, Order};
use crate::odf_transactions::old_booked_payments;
use anyhow::{anyhow, Result};
use failure::Fail;
use itertools::{structs::GroupBy, Itertools};

// pub fn run(config: Config){
pub fn run(config: Config) -> Result<()> {
    let start_date = NaiveDate::from_ymd(2019, 01, 01);
    let date_to_payment = ebanking_payments()?;
    let date_to_old_payments = old_booked_payments(&start_date)?;
    let input_filename = &config.input_filename;
    let mut of = File::create(&config.output_filename)?;
    println!("; {} -> {}", input_filename, &config.output_filename);
    writeln!(of, "; {:?}\n", input_filename)?;
    let messages = parse_messages(&input_filename)?;

    first_after(&start_date, &messages)
        .map(|message| {
            writeln!(of, "{}\n", Transaction::new_opening_balance(message))
        })
        .unwrap_or(Ok(()))?;

    let grouped = &stmtlines_grouped_by_date_after(&start_date, &messages);
    for (date, stmtlines_group) in grouped {
        // TODO match with old_payments
        // TODO map sum to old payment
        // TODO collect old payments that don't match new ones,
        // TODO try to match them to ebill payments
        let mut unmatched_o = date_to_old_payments.get(&date);
        for stmtline in stmtlines_group {
            // TODO discard unambiguous sums
            let amount = stmtline.amount;
            if let Some(ref mut unmatched) = unmatched_o {
                let pos_o = unmatched.iter().position(|(a, _)| {
                    // AAAAAAAAAAAAAAAAAAAARRRRRRRRRRRRGH
                    amount
                        .to_f32()
                        .and_then(|a32| a32.to_f64())
                        .map(|a64| &a64 == a)
                        .unwrap_or(false)
                });
                if let Some(pos) = pos_o {
                    // WTFFFFFFFFFFFFFFFFFF
                    unmatched
                        .into_iter()
                        .nth(pos)
                        .and_then(|pmt| {
                            Some(writeln!(&mut of, "; old pmt {:?}", pmt))
                        })
                        .unwrap_or(Ok(()))?;
                }
            }
            if details_match(stmtline, &R_GROUPED_EBANKING) {
                write_grouped_ebanking_orders(
                    &mut of,
                    stmtline,
                    &config,
                    &date_to_payment,
                )?;
            } else {
                if details_match(stmtline, &R_GROUPED_EBILL) {
                    let p_count = date_to_old_payments
                        .get(&date)
                        .map(|ps| ps.len())
                        .unwrap_or(0);
                    writeln!(&mut of, "; ebill group of {}", p_count)?;
                }
                write_stmtline(&mut of, stmtline, &config)?;
            }
        }
        unmatched_o
            .and_then(|unm| {
                Some(writeln!(&mut of, "; unmatched {:?}", unm))
            })
            .unwrap_or(Ok(()))?;
    }
    Ok(())
}

fn details_match(stmtline: &StatementLine, regex: &Regex) -> bool {
    let cls = |details: &String| regex.is_match(details);
    stmtline
        .supplementary_details
        .as_ref()
        .map(cls)
        .unwrap_or(false)
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

pub fn stmtlines_grouped_by_date_after<'a>(
    start_date: &'a NaiveDate,
    messages: &'a [Message],
) -> GroupBy<
    &'a NaiveDate,
    impl Iterator<Item = &'a StatementLine>,
    // A closure that does not move, borrow,
    // or otherwise access (capture) local variables
    // is coercable to a function pointer (fn).
    // (The other solution would be to box the closure
    // like this: .group_by(Box::new(|s| &s.value_date))
    // and declare this type instead of fn:
    // Box<dyn Fn(& &'a StatementLine) -> &'a NaiveDate>)
    fn(&&'a StatementLine) -> &'a NaiveDate,
> {
    stmtlines_after(start_date, messages).group_by(|s| &s.value_date)
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

fn write_grouped_ebanking_orders(
    of: &mut File,
    stmtline: &StatementLine,
    config: &Config,
    date_to_payment: &HashMap<NaiveDate, Vec<Order>>,
) -> Result<()> {
    let date = &stmtline.value_date;
    let all_orders = date_to_payment
        .get(date)
        .ok_or(anyhow!("no payments for grouped ebanking"))?;
    let chf_orders: Vec<_> =
        all_orders.iter().filter(|p| p.is_chf).collect();
    let sum = chf_orders.iter().map(|s| s.amount).sum::<Decimal>();
    let total_count = chf_orders.len();
    write!(of, "; {} grouped ebanking of {} (sum)\n", sum, total_count)?;
    for (count, order) in chf_orders.iter().enumerate() {
        // write!(of, "; {} {}\n", order.amount, &order.description)?;
        let recipient =
            determine_recipient(&order.description, config.interactive)?;
        writeln!(
            of,
            "; {} of {}\n{}",
            count + 1,
            total_count,
            Transaction::from_order(order, recipient),
        )?;
    }
    if sum != stmtline.amount {
        return Err(anyhow!(
            "orders sum {} != {} aggregate statement amount",
            sum,
            stmtline.amount
        ));
    }
    Ok(())
}

fn write_stmtline(
    output_file: &mut File,
    statement: &StatementLine,
    config: &Config,
) -> Result<()> {
    let owner_info = extract_info_to_owner(statement).unwrap_or("");
    let recipient = determine_recipient(owner_info, config.interactive)?;
    let transaction = Transaction::new(statement, recipient);
    writeln!(output_file, "{}\n", transaction)?;
    Ok(())
}

fn determine_recipient(
    description: &str,
    interactive: bool,
) -> Result<Recipient> {
    let mut recipient = extract_recipient(description);
    if interactive {
        recipient = change_account_interactively(&recipient, description)?;
    }
    Ok(recipient)
}

fn change_account_interactively(
    recipient: &Recipient,
    owner_info: &str,
) -> Result<Recipient> {
    let init_acc = recipient.account;
    let account_o = choose_account_from_command_line(init_acc, owner_info);
    let account = account_o?;
    println!(";;{}\n", account);
    Ok(Recipient {
        name: String::from(owner_info),
        account,
    })
}

struct Transaction<'a> {
    recipient: Recipient,
    details: Option<&'a str>,
    info_to_owner: Option<&'a str>,
    date: &'a NaiveDate,
    amount: Decimal,
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
            amount: message.opening_balance.amount,
            recipient,
            details: Option::None,
            info_to_owner: info_to_owner.as_ref().map(String::as_str),
        }
    }
    fn from_order(
        order: &'a Order,
        recipient: Recipient,
    ) -> Transaction<'a> {
        Transaction {
            date: &order.date,
            recipient,
            info_to_owner: Some(&order.description),
            amount: order.amount,
            details: None,
        }
    }
    fn new(
        statement: &'a mt940::StatementLine,
        recipient: Recipient,
    ) -> Transaction<'a> {
        let entry_date = statement.entry_date.as_ref();
        let date = entry_date.unwrap_or(&statement.value_date);
        let info_to_owner = extract_info_to_owner(statement);
        Transaction {
            date,
            recipient,
            info_to_owner,
            amount: statement.amount,

            details: statement
                .supplementary_details
                .as_ref()
                .map(String::as_str),
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
            "{} {}
  ; {}
  ; {}
  {}             {:?}
  Assets::Checking",
            self.date.format("%Y/%m/%d"),
            remove_newlines(&self.recipient.name),
            details_str,
            remove_newlines(info_to_owner_str),
            self.recipient.account,
            self.amount
        )
    }
}

fn extract_info_to_owner(
    statement: &mt940::StatementLine,
) -> Option<&str> {
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
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn determine_recipient_maestro() {
        let desc = "?ZKB:2218 Einkauf ZKB Maestro Karte Nr. 73817865, LE; POUSSE-POUSSE SARL 1205";
        let recipient = determine_recipient(desc, false).unwrap();
        println!("|{}|", recipient.name);
        assert!(recipient.name == "LE; POUSSE-POUSSE SARL 1205");
    }
}
