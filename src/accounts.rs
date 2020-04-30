
#[derive(Debug)]
pub enum Appartment {
    Electricity,
    Rent,
}

#[derive(Debug)]
enum Expense {
    Maestro,
    Rest,
    Appartment(Appartment),
}

#[derive(Debug)]
enum Equity {
    OpeningBalances,
}

#[derive(Debug)]
enum Account {
    Expense(Expense),
    Equity(Equity),
}
