extern crate derive_more;
use derive_more::Display;

use crossterm::{
    event::{
        read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
    Result,
};
use std::io::{stdout, Write};
use std::{cmp, fmt};

#[derive(Debug, Clone, Copy, Display, PartialEq)]
pub enum Apartment {
    Electricity,
    Rent,
}

#[derive(Debug, Clone, Copy, Display, PartialEq)]
pub enum Expenses {
    Maestro,
    Rest,
    #[display(fmt = "Apartement::{}", _0)]
    Apartment(Apartment),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Equity {
    OpeningBalances,
}

impl<'a> fmt::Display for Equity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Opening Balances")
    }
}

#[derive(Debug, Clone, Copy, Display, PartialEq)]
pub enum Account {
    #[display(fmt = "Expenses::{}", _0)]
    Expenses(Expenses),
    #[display(fmt = "Equity::{}", _0)]
    Equity(Equity),
}

pub const ACCOUNTS: [Account; 3] = [
    Account::Expenses(Expenses::Maestro),
    Account::Expenses(Expenses::Apartment(Apartment::Electricity)),
    Account::Expenses(Expenses::Rest),
];

pub fn choose_account_from_command_line<'a>(initial_account: Account) -> Result<Account> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnableMouseCapture)?;
    let mut selected: usize = 0;
    let found_index_o =  ACCOUNTS.iter().position(|&a| a == initial_account);
    if let Some(found_index) = found_index_o {
        selected = found_index;
    }
    loop {
        //https://www.key-shortcut.com/en/writing-systems/35-symbols/arrows/
        println!("Choose an account ⯅⯆ ⮠");
        for (index, account) in ACCOUNTS.iter().enumerate() {
            let cursor = if index == selected { '⯈' } else { ' ' };
            println!("{} {} {}", cursor, index, account);
        }
        // Blocking read
        let event = read()?;
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL
            }) => {
               panic!("user chose ctrl-c")
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(value),
                ..
            }) => {
                if let Some(val_dig) = value.to_digit(10) {
                    if (val_dig as usize) < ACCOUNTS.len() && val_dig > 0 {
                        selected = val_dig as usize
                    }
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Up, ..
            }) => {
                selected = cmp::max(0, selected - 1);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                ..
            }) => {
                selected = cmp::min(ACCOUNTS.len() - 1, selected + 1);
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
