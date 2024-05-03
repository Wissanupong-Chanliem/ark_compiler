mod tokenizer;
mod arkparser;
mod ctranspiler;
use clap::{builder::OsStr, Parser};
use std::{fs, io::Write, path::{Path, PathBuf}};
use ctranspiler::gen_c_program;

#[derive(Parser)]
struct Cli{
    source:PathBuf,
}
fn main(){
    let args = Cli::parse();
    let source_code = match fs::read_to_string(args.source.clone()){
        Ok(code)=>code,
        Err(_)=>panic!("Unable to find your source code"),
    };
    let mut parser = arkparser::ArkParser::new(source_code.as_str());
    let mut bin_location = match std::env::current_exe(){
        Ok(path) => path,
        Err(_) => panic!("can't find bin path"),
    };
    bin_location.pop();
    //let mut source_name = args.source.file_stem().unwrap();
    bin_location.push([String::from("temp_") , args.source.file_stem().unwrap().to_str().unwrap().to_string(),String::from(".c")].concat());
    
    println!("{:?}",bin_location);
    let mut file = fs::File::create(bin_location).unwrap();
    let _ = file.write(gen_c_program(&parser.parse()).as_bytes());
    
}
