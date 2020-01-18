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
        let recipient =
            extract_maestro_recipient(statement).unwrap_or("not maestro");
        let owner_info =
            statement.information_to_account_owner.as_ref()
            .map(String::as_ref).unwrap_or("no info");
        println!("{} {}\n {:?}",
                 statement.value_date.format("%Y/%m/%d").to_string(),
                 statement.supplementary_details.as_ref().unwrap(),
                 statement.amount,);
        println!("owner info: {}", owner_info);
        println!("maestro recipient: {}\n", remove_newlines(recipient));
    }
        
}

fn extract_maestro_recipient(statement: &mt940::StatementLine)-> Option<&str>{
    let op_string_ref: Option<&String> =
        statement.information_to_account_owner.as_ref();
    // https://stackoverflow.com/questions/31233938/converting-from-optionstring-to-optionstr
    let op_str: Option<&str> = op_string_ref.map(String::as_str);
    op_str.and_then(|str|{
        extract_maestro_recipient_from_owner_info(str)
    })
}

fn extract_maestro_recipient_from_owner_info(owner_info: &str) -> Option<&str>{
    // https://rust-lang-nursery.github.io/rust-cookbook/text/regex.html
    lazy_static! {
        static ref RECIPIENT: Regex = Regex::new(
            r"(?s).*Einkauf ZKB Maestro Karte Nr. 73817865[^,]*,(.*$)").unwrap();
        static ref NEW_LINE: Regex = Regex::new(r"\n").unwrap();
    }
    let c = RECIPIENT.captures(owner_info);
    c.and_then(|cap|{
         cap.get(1).map(|m| m.as_str())
    })
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
