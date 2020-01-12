use std::env;
use geldparser::Config;
    
fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args);
    geldparser::run(config);
}

