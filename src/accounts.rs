extern crate derive_more;
extern crate lazy_static;
use derive_more::Display;
use regex::Regex;

use crossterm::{
    cursor::MoveTo,
    event::{
        read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode,
        KeyEvent, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    Result,
};
use std::fmt;
use std::io::{stdout, Write};

use Account::*;
use Eapartment::*;
use Eexpenses::*;

#[derive(Debug, Clone, Copy, Display, PartialEq)]
pub enum Eapartment {
    Electricity,
    Rent,
}

#[derive(Debug, Clone, Copy, Display, PartialEq)]
pub enum Eexpenses {
    Maestro,
    Rest,
    #[display(fmt = "Apartement::{}", _0)]
    Apartment(Eapartment),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Eequity {
    OpeningBalances,
}

impl<'a> fmt::Display for Eequity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Opening Balances")
    }
}

#[derive(Debug, Clone, Copy, Display, PartialEq)]
pub enum Account {
    #[display(fmt = "Expenses::{}", _0)]
    Expenses(Eexpenses),
    #[display(fmt = "Equity::{}", _0)]
    Equity(Eequity),
}

pub const ACCOUNTS: [Account; 3] = [
    Expenses(Maestro),
    Expenses(Apartment(Electricity)),
    Expenses(Rest),
];

lazy_static! {
    pub static ref R_GROUPED_EBILL: Regex =
        Regex::new(r"eBill +\(\d+\)").unwrap();
    pub static ref R_GROUPED_EBANKING: Regex =
        Regex::new(r"eBanking +\(\d+\)").unwrap();
    static ref M_MAESTRO: Matcher<'static> = m1(
        Expenses(Maestro),
        // (?s) allow . to match \n
        r"(?s).*Einkauf ZKB Maestro Karte Nr. 73817865[^,]*, ?(.*$)"
    );
    static ref M_SIG: Matcher<'static> = m1(
        Expenses(Apartment(Electricity)),
        r"(Services Industriels de Geneve)"
    );
    static ref M_SINGLE_EBANKING: Matcher<'static> = m1(
        Expenses(Rest),
        r"(?s)\?ZKB:2214(.*)Gemaess Ihrem eBanking Auftrag.*"
    );
    pub static ref MATCHERS: Vec<&'static Matcher<'static>> =
        vec![&M_MAESTRO, &M_SIG, &M_SINGLE_EBANKING];
}

pub struct Matcher<'a> {
    account: Account,
    regex: Regex,
    name_template: &'a str,
}

pub struct Recipient {
    pub name: String,
    pub account: Account,
}

fn m<'a>(
    account: Account,
    regex_str: &'a str,
    name_template: &'a str,
) -> Matcher<'a> {
    Matcher {
        account,
        regex: Regex::new(regex_str).unwrap(),
        name_template,
    }
}

fn m1<'a>(account: Account, regex_str: &'a str) -> Matcher<'a> {
    m(account, regex_str, "$1")
}

impl<'a> Matcher<'a> {
    pub fn match_to_recipient(
        &self,
        owner_info: &str,
    ) -> Option<Recipient> {
        self.regex.captures(owner_info).and_then(|cap| {
            let mut name = String::from("");
            cap.expand(self.name_template, &mut name);
            Some(Recipient {
                name,
                account: self.account,
            })
        })
    }
}

fn rest_recipient(owner_info: &str) -> Recipient {
    Recipient {
        name: String::from(owner_info),
        account: Expenses(Rest),
    }
}

pub fn extract_recipient(owner_info: &str) -> Recipient {
    MATCHERS
        .iter()
        .find_map(|m| m.match_to_recipient(owner_info))
        .unwrap_or_else(|| rest_recipient(owner_info))
}

pub fn choose_account_from_command_line<'a>(
    initial_account: Account,
    owner_info: &str,
) -> Result<Account> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnableMouseCapture)?;
    let mut selected: usize = 0;
    let found_index_o =
        ACCOUNTS.iter().position(|&a| a == initial_account);
    if let Some(found_index) = found_index_o {
        selected = found_index;
    }
    loop {
        execute!(stdout, Clear(ClearType::All), MoveTo(0, 9))?;
        //https://www.key-shortcut.com/en/writing-systems/35-symbols/arrows/
        println!("Choose an account ⯅⯆ ⮠");
        for (index, account) in ACCOUNTS.iter().enumerate() {
            let cursor = if index == selected { ">" } else { " " };
            println!("{} {} {}", cursor, index, account);
        }
        execute!(stdout, MoveTo(0, 0))?;
        println!("{}", owner_info);
        println!("\n{}", ACCOUNTS[selected]);
        // Blocking read
        let event = read()?;
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
            }) => panic!("user chose ctrl-c"),
            Event::Key(KeyEvent {
                code: KeyCode::Char(value),
                ..
            }) => {
                if let Some(val_dig) = value.to_digit(10) {
                    if (val_dig as usize) < ACCOUNTS.len() {
                        selected = val_dig as usize
                    }
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Up, ..
            }) if selected > 0 => {
                selected = selected - 1;
                println!("selected {}", selected);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                ..
            }) if selected < ACCOUNTS.len() - 1 => {
                selected = selected + 1;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            }) => break,
            _ => {
                println!("hohih");
            }
        }
    }
    execute!(stdout, DisableMouseCapture)?;
    disable_raw_mode()?;

    Ok(ACCOUNTS[selected])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_to_maestro_recipient() {
        let desc = "?ZKB:2218 Einkauf ZKB Maestro Karte Nr. 73817865, LE; POUSSE-POUSSE SARL 1205";
        let recipient = M_MAESTRO.match_to_recipient(desc).unwrap();
        println!("|{}|", recipient.name);
        assert!(recipient.account == Expenses(Maestro));
        assert!(recipient.name == "LE; POUSSE-POUSSE SARL 1205");
    }

    #[test]
    fn extract_maestro_recipient() {
        let desc = "?ZKB:2218 Einkauf ZKB Maestro Karte Nr. 73817865, LE; POUSSE-POUSSE SARL 1205";
        let recipient = extract_recipient(desc);
        println!("|{}|", recipient.name);
        assert!(recipient.account == Expenses(Maestro));
        assert!(recipient.name == "LE; POUSSE-POUSSE SARL 1205");
    }

    #[test]
    fn match_to_single_ebanking_recipient() {
        let desc = "?ZKB:2214 IV Mad\nAvenida\nES-28023 Madrid\nmaintenance\nGemaess Ihrem eBanking Auftrag BL---";
        let recipient =
            M_SINGLE_EBANKING.match_to_recipient(desc).unwrap();
        println!("|{}|", recipient.name);
        assert!(recipient.account == Expenses(Rest));
        assert!(
            recipient.name
                == " IV Mad\nAvenida\nES-28023 Madrid\nmaintenance\n"
        );
    }
}
