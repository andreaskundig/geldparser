use std::fmt;

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
pub enum Equity {
    OpeningBalances,
}

impl<'a> fmt::Display for Equity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Opening Balances")
    }
}

#[derive(Debug)]
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
