use crossterm::{
    event::{read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
    Result,
};
use std::io::{stdout, Write};
use std::{cmp, fmt};

#[derive(Debug, Copy, Clone)]
pub enum Apartment {
    Electricity,
    Rent,
}

impl<'a> fmt::Display for Apartment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Apartment::Electricity => write!(f, "Electricity"),
            Apartment::Rent => write!(f, "Rent"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Expenses {
    Maestro,
    Rest,
    Apartment(Apartment),
}

impl<'a> fmt::Display for Expenses {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expenses::Maestro => write!(f, "Maestro"),
            Expenses::Rest => write!(f, "Rest"),
            Expenses::Apartment(a) => write!(f, "Apartement::{}", a),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Equity {
    OpeningBalances,
}

impl<'a> fmt::Display for Equity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Opening Balances")
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Account {
    Expenses(Expenses),
    Equity(Equity),
}

impl<'a> fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Account::Expenses(e) => write!(f, "Expenses::{}", e),
            Account::Equity(e) => write!(f, "Equity::{}", e),
        }
    }
}

pub const ACCOUNTS: [Account; 3] = [
    Account::Expenses(Expenses::Maestro),
    Account::Expenses(Expenses::Apartment(Apartment::Electricity)),
    Account::Expenses(Expenses::Rest),
];

pub fn choose_account_from_command_line() -> Result<Account> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnableMouseCapture)?;
    let mut selected: usize = 0;
    loop {
        println!("Choose an account ↓↑↲");
        for (index, account) in ACCOUNTS.iter().enumerate() {
            let selected = index == selected;
            let cursor = if selected { '>' } else { ' ' };
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

    Ok(ACCOUNTS[selected])
}
