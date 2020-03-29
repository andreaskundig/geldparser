//! Demonstrates how to block read events.
//!
//! cargo run --example event-read

use std::cmp;
use std::io::{stdout, Write};

use crossterm::{
    cursor::position,
    event::{read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
    Result,
};

const HELP: &str = r#"Blocking read()
 - Keyboard, mouse and terminal resize events enabled
 - Hit "c" to print current cursor position
 - Use Esc to quit
"#;

const ACCOUNTS: [&str; 3] = [
    "Expenses::Maestro",
    "Expenses::Appartment::Electricity",
    "Expenses::Rest",
];

fn print_events() -> Result<()> {
    let mut selected: usize = 0;
    loop {
        for (index, account) in ACCOUNTS.iter().enumerate() {
            let selected = index == selected;
            let cursor = if selected { '>' } else { ' ' };
            println!("{} {} {}", cursor, index, account);
        }

        // Blocking read
        let event = read()?;

        println!("Event: {:?} {}\r", event, selected);

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

        if event == Event::Key(KeyCode::Char('q').into()) {
            break;
        }
        if event == Event::Key(KeyCode::Esc.into()) {
            break;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    println!("{}", HELP);

    enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(stdout, EnableMouseCapture)?;

    if let Err(e) = print_events() {
        println!("Error: {:?}\r", e);
    }

    execute!(stdout, DisableMouseCapture)?;

    disable_raw_mode()
}
