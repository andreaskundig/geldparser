#[macro_use] extern crate lazy_static;
extern crate regex;

use mt940::parse_mt940;
use mt940::sanitizers;
use std::fs;
use regex::Regex;
use std::borrow::Cow;

pub fn run(config: Config){
    let filename = config.filename;
    println!("{:?}", filename);

    let contents = fs::read_to_string(filename)
        .expect("Something went wrong reading the file");

    let sanitized = sanitizers::sanitize(&contents[..]);
    let messages = parse_mt940(&sanitized[..]).unwrap();
    let message = &messages[0];
    println!("{:?}", message.opening_balance.amount);
    //let statement = &message.statement_lines[0];
    for statement in &message.statement_lines {
        println!("{} {}\n {:?}",
                 statement.value_date.format("%Y/%m/%d").to_string(),
                 statement.supplementary_details.as_ref().unwrap(),
                 statement.amount,);
        let entry_o = extract_transaction(statement);
        if entry_o.is_some() {
             let entry = entry_o.unwrap();
            println!("recipient: {}",
                     remove_newlines(entry.recipient));
             println!("account: {}", entry.account);
        }
        let owner_info =
            statement.information_to_account_owner.as_ref()
            .map(String::as_ref).unwrap_or("no info");
        println!("owner info: {}", owner_info);
        println!("------------------------");
    }
        
}

// TODO match regex, return Entry (find a better name?)
struct Transaction<'a>{
    recipient : &'a str,
    account : &'a str,
}

fn extract_transaction(statement: &mt940::StatementLine)-> Option<Transaction>{
    let owner_info = extract_owner_info(statement);
    owner_info.and_then(
        {|oi|
         extract_maestro_transaction(oi)
         .or(extract_sig_transaction(oi))
        })
        
}

fn extract_owner_info(statement: &mt940::StatementLine)-> Option<&str>{
    let op_string_ref: Option<&String> =
        statement.information_to_account_owner.as_ref();
    // https://stackoverflow.com/questions/31233938/converting-from-optionstring-to-optionstr
    op_string_ref.map(String::as_str)
}

fn extract_maestro_transaction(owner_info: &str) -> Option<Transaction>{
    // https://rust-lang-nursery.github.io/rust-cookbook/text/regex.html
    lazy_static! {
        static ref MAESTRO: Regex = Regex::new(
            r"(?s).*Einkauf ZKB Maestro Karte Nr. 73817865[^,]*,(.*$)").unwrap();
    }
    let c = MAESTRO.captures(owner_info);
    c.and_then(|cap|{
        cap.get(1).map(
            |m| Transaction{recipient: m.as_str(),
                            account: "debit-card"})
    })
}

fn extract_sig_transaction(owner_info: &str) -> Option<Transaction>{
    // https://rust-lang-nursery.github.io/rust-cookbook/text/regex.html
    lazy_static! {
        static ref SIG: Regex = Regex::new(
            r"Services Industriels de Geneve").unwrap();
    }
    if SIG.is_match(owner_info) {
        Some(Transaction{recipient: "Services Industriels de Geneve",
                         account: "loyer"})
    }else{
        None
    }
    
}
//TODO understand cow
fn remove_newlines(text: &str) -> Cow<str>{
    // https://rust-lang-nursery.github.io/rust-cookbook/text/regex.html
    lazy_static! {
        static ref NEWLINE: Regex = Regex::new(r"\n").unwrap();
    }
    NEWLINE.replace_all(text, "; ")
}

pub struct Config {
    pub filename: String,
}

impl Config {
    pub fn new(args: &[String]) -> Config {
        let mut filename = String::from("../bewegungen/2019.mt940");
        if args.len() > 1 {
            filename = args[1].clone();
        }
        Config { filename }
    }
}
