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
use rust_decimal::Decimal;
use std::{
    borrow::Cow, collections::HashMap, fmt, fs, fs::File, io::prelude::*,
};
pub mod accounts;
pub mod csv_orders;
pub mod odf_transactions;
use crate::csv_orders::{ebanking_payments_from_csvs, Order};
use crate::odf_transactions::old_booked_payments;
use anyhow::{anyhow, Result};
use failure::Fail;
use itertools::{structs::GroupBy, Itertools};

// pub fn run(config: Config){
pub fn run(config: Config) -> Result<()> {
    let start_date = NaiveDate::from_ymd(2019, 01, 01);
    // data from bank for disaggregating ebanking payments
    let date_to_payment = ebanking_payments_from_csvs()?;
    // data from my records to help disaggregate ebill payments
    let mut date_to_old_payments = old_booked_payments(&start_date)?;
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
        // group contains all stmtlines for a date

        let mut processed_transactions = Vec::new();
        let mut ebill_stmtlines = Vec::new();
        for stmtline in stmtlines_group {
            if details_match(stmtline, &R_GROUPED_EBANKING) {
                // ebanking payments are disaggregated with data from csvs
                let found_transactions =
                    transactions_from_grouped_ebanking_orders(
                        stmtline,
                        &config,
                        &date_to_payment,
                    )?;
                write_ebanking_transactions(&mut of, &found_transactions)?;
                processed_transactions.extend(found_transactions);
            } else if details_match(stmtline, &R_GROUPED_EBILL) {
                // ebills need to be disaggregated later

                ebill_stmtlines.push(stmtline);
            } else {
                let t = transaction_from_stmtline(stmtline, &config)?;
                writeln!(&mut of, "{}\n", t)?;
                processed_transactions.push(t);
            }
        }

        // payments from the old csv file
        let old_payments_o = date_to_old_payments.get_mut(&date);
        if let Some(old_payments) = old_payments_o {
            // try to match processed transactions to old_payments_o
            // remove unambiguous matches
            for processed_transaction in processed_transactions {
                let amount = &processed_transaction.amount;
                let matching =
                    old_payments.iter().filter(|(a, _)| a == amount);
                if matching.count() == 1 {
                    // discard the matching payment
                    let old_pmt_o = old_payments
                        .iter()
                        .position(|(a, _)| amount == a)
                        .and_then(|pos| Some(old_payments.remove(pos)));
                    if let Some(op) = old_pmt_o {
                        writeln!(
                            &mut of,
                            "; already processed old pmt on {}: {:?}",
                            date, op
                        )?;
                    }
                }
            }

            for ebill_stmtline in ebill_stmtlines {
                // use the ebill regex to extract the number of paymentss
                let dets = ebill_stmtline
                    .supplementary_details
                    .as_ref()
                    .ok_or(anyhow!("no supplementary details"))?;

                let pmt_count = R_GROUPED_EBILL
                    .captures(dets)
                    .map(|cap| cap.get(1).map(|mtch| mtch.as_str()))
                    .flatten()
                    .ok_or(anyhow!("no ebill count in '{}'", dets))?
                    .parse::<u32>()?;

                let target_sum = ebill_stmtline.amount;
                //TODO extract payment count and sum from ebill_stmtlines
                // if both match unambiguously with old payments
                // write the disaggregated payments to output
                writeln!(
                    &mut of,
                    "; ebill ({})({}) for {} {:?}\n",
                    pmt_count,
                    old_payments.len(),
                    date,
                    old_payments
                )?;
                let t =
                    transaction_from_stmtline(ebill_stmtline, &config)?;
                writeln!(&mut of, "{}\n", t)?;
            }
        } else {
            // nothing to match
            for ebill_stmtline in ebill_stmtlines {
                let t =
                    transaction_from_stmtline(ebill_stmtline, &config)?;
                writeln!(&mut of, "{}\n", t)?;
            }
        }
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

fn transactions_from_grouped_ebanking_orders<'a>(
    stmtline: &StatementLine,
    config: &Config,
    date_to_payment: &'a HashMap<NaiveDate, Vec<Order>>,
) -> Result<Vec<Transaction<'a>>> {
    let date = &stmtline.value_date;
    let all_orders = date_to_payment
        .get(date)
        .ok_or(anyhow!("no payments for grouped ebanking"))?;
    let chf_orders: Vec<_> =
        all_orders.iter().filter(|p| p.is_chf).collect();
    let sum = chf_orders.iter().map(|s| s.amount).sum::<Decimal>();
    let transactions = chf_orders
        .iter()
        .map(|&order| {
            let recipient = determine_recipient(
                &order.description,
                config.interactive,
            )?;
            Ok(Transaction::from_order(order, recipient))
        })
        .collect::<Result<Vec<Transaction>>>()?;
    if sum != stmtline.amount {
        return Err(anyhow!(
            "orders sum {} != {} aggregate statement amount",
            sum,
            stmtline.amount
        ));
    }
    Ok(transactions)
}

fn write_ebanking_transactions<'a>(
    of: &mut File,
    transactions: &Vec<Transaction<'a>>,
) -> Result<()> {
    let sum = transactions.iter().map(|t| t.amount).sum::<Decimal>();
    let total_count = transactions.len();
    write!(of, "; {} grouped ebanking of {} (sum)\n", sum, total_count)?;
    for (count, transaction) in transactions.iter().enumerate() {
        // write!(of, "; {} {}\n", order.amount, &order.description)?;
        writeln!(
            of,
            "; {} of {}\n{}",
            count + 1,
            total_count,
            transaction
        )?;
    }
    Ok(())
}

fn transaction_from_stmtline<'a>(
    statement: &'a StatementLine,
    config: &Config,
) -> Result<Transaction<'a>> {
    let owner_info = extract_info_to_owner(statement).unwrap_or("");
    let recipient = determine_recipient(owner_info, config.interactive)?;
    let transaction = Transaction::new(statement, recipient);
    Ok(transaction)
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
