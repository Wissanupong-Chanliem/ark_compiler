
mod arkparser;
use clap::Parser;
use std::{fs, path::PathBuf};

#[derive(Parser)]
struct Cli{
    source:PathBuf,
}
fn main(){
    let args = Cli::parse();
    let source_code = match fs::read_to_string(args.source){
        Ok(code)=>code,
        Err(_)=>panic!("Unable to find your source code"),
    };
    let mut parser = arkparser::ArkParser::new(source_code.as_str());
    parser.parse();
    
}
