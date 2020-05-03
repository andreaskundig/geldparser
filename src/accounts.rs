extern crate derive_more;
use derive_more::{Display};

use crossterm::{
    event::{read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
    Result,
};
use std::io::{stdout, Write};
use std::{cmp, fmt};

#[derive(Debug, Display)]
pub enum Apartment {
    Electricity,
    Rent,
}

// impl<'a> fmt::Display for Apartment {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match *self {
//             Apartment::Electricity => write!(f, "Electricity"),
//             Apartment::Rent => write!(f, "Rent"),
//         }
//     }
// }

#[derive(Debug, Display)]
pub enum Expenses<'a> {
    Maestro,
    Rest,
    #[display(fmt = "Apartement::{}", _0)]
    Apartment(&'a Apartment),
}

#[derive(Debug)]
pub enum Equity {
    OpeningBalances,
}

impl<'a> fmt::Display for Equity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Opening Balances")
    }
}

#[derive(Debug, Display)]
pub enum Account<'a> {
    #[display(fmt = "Expenses::{}", _0)]
    Expenses(&'a Expenses<'a>),
    #[display(fmt = "Equity::{}", _0)]
    Equity(&'a Equity),
}

pub const ACCOUNTS: [Account; 3] = [
    Account::Expenses(&Expenses::Maestro),
    Account::Expenses(&Expenses::Apartment(&Apartment::Electricity)),
    Account::Expenses(&Expenses::Rest),
];

pub fn choose_account_from_command_line<'a> () -> Result<&'a Account<'a>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnableMouseCapture)?;
    let mut selected: usize = 0;
    loop {
        //https://www.key-shortcut.com/en/writing-systems/35-symbols/arrows/
        println!("Choose an account ⯅⯆ ⮠");
        for (index, account) in ACCOUNTS.iter().enumerate() {
            let cursor = if index == selected { '⯈' } else { ' ' };
            println!("{} {} {}", cursor, index, account);
        }
        // Blocking read
        let event = read()?;
        if let Event::Key(KeyEvent {
            code: KeyCode::Char(value),
            ..
        }) = event
        {
            if let Some(val_dig) = value.to_digit(10) {
                if (val_dig as usize) < ACCOUNTS.len() && val_dig > 0 {
                    selected = val_dig as usize;
                }
            }
        }
        if event == Event::Key(KeyCode::Up.into()) {
            selected = if selected > 0 { selected - 1 } else { selected }
        }
        if event == Event::Key(KeyCode::Down.into()) {
            selected = cmp::min(ACCOUNTS.len() - 1, selected + 1);
        }
        if event == Event::Key(KeyCode::Enter.into()) {
            break;
        }
    }
    execute!(stdout, DisableMouseCapture)?;
    disable_raw_mode()?;

    Ok(&ACCOUNTS[selected])
}
