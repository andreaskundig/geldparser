use mt940::parse_mt940;
use mt940::sanitizers;
use std::fs;

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
        println!("{:?}",
                 statement.information_to_account_owner
                 );
    }
        
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
