mod lexer;
use clap::Parser;
use std::{fs, path::PathBuf, io::Read};

#[derive(Parser)]
struct Cli{
    source:PathBuf,
}
fn main(){
    let args = Cli::parse();
    let mut source_code = match fs::read_to_string(args.source){
        Ok(code)=>code,
        Err(error)=>panic!("Unable to find your source code"),
    };
    for line in source_code.lines(){

    }
}
