mod tokenizer;
mod arkparser;
mod ctranspiler;
mod semantic_analyzer;
mod symbol_table;
use clap::{builder::OsStr, Parser};
use semantic_analyzer::SemanticAnalyzer;
use symbol_table::SymbolTable;
use tokenizer::Tokenizer;
use arkparser::ArkParser;
use std::{cell::RefCell, fs, io::Write, path::{Path, PathBuf}, rc::Rc};
use colored::Colorize;
#[derive(Parser)]
struct Cli{
    source:PathBuf,
}

#[derive(Clone)]
struct CompilerError{
    error_type:ErrorType,
    error_message:String,
    pos: (u32,u32),
    length: u32,
}


impl CompilerError{
    pub fn new(error_type:ErrorType,error_message:&str,error_pos:(u32,u32),length:u32) -> CompilerError{
        CompilerError {
            error_type,
            error_message:String::from(error_message),
            pos:error_pos,
            length
        }
    }
}
#[derive(Debug,Clone, Copy)]
pub enum ErrorType {
    SemanticError,
    SyntaxError,
    LexicalError
}

impl ErrorType{
    pub fn as_str(&self) -> &str{
        match self {
            ErrorType::SemanticError => "Sematic Error",
            ErrorType::SyntaxError => "Syntax Error",
            ErrorType::LexicalError => "Lexical Error"
        }
    }
}

struct ErrorPipeline{
    error_generated:RefCell<Vec<CompilerError>>
}

impl ErrorPipeline {
    pub fn raise_error(&self,error_type:ErrorType,error_message:&str,error_pos:(u32,u32),length:u32){
        self.error_generated.borrow_mut().push(
            CompilerError {
                error_type,
                error_message:String::from(error_message),
                pos:error_pos,
                length
            }
        )
    }
    pub fn report_error(&self,err:CompilerError){
        self.error_generated.borrow_mut().push(
            err
        )
    }
}

fn main(){
    let args = Cli::parse();
    let source_code = match fs::read_to_string(args.source.clone()){
        Ok(code)=>code,
        Err(_)=>panic!("Unable to find your source code"),
    };
    let symbol_table = Rc::new(SymbolTable::new(symbol_table::Scope::Global));
    let error_pipe = ErrorPipeline {error_generated:RefCell::new(vec![])} ;
    let mut tokenizer = Tokenizer::new(source_code.as_str(), &error_pipe);
    let mut parser = ArkParser::new(&mut tokenizer,&error_pipe);
    
    // let mut bin_location = match std::env::current_exe(){
    //     Ok(path) => path,
    //     Err(_) => panic!("can't find bin path"),
    // };
    match parser.parse() {
        Some(program) => {
            println!("{:#?}",program.instructions);
            let semantic_analyzer = SemanticAnalyzer::new(&program, &error_pipe,Rc::clone(&symbol_table));
            semantic_analyzer.analyze();
            
            println!("{:#?}",symbol_table);
        },
        None => {
            let source_lines = source_code.split('\n').collect::<Vec<&str>>();
            for e in error_pipe.error_generated.borrow().clone().into_iter() {
                let size = e.pos.0.to_string().len();
                let mut space = String::new();
                for _ in 0..size {
                    space += " ";
                }
                let source_snippet = source_lines[(e.pos.0-1) as usize];
                let trimed_snippet = source_snippet.trim();
                let error_col = (e.pos.1-(source_snippet.len()-trimed_snippet.len()) as u32 -1) as usize;
                println!(
                    "{}: {}",
                    e.error_type.as_str().red().bold(),
                    e.error_message.white().bold()
                );
                println!(
                    "{}--> {}:{}:{}",
                    space,
                    args.source.clone().into_os_string().into_string().unwrap(),
                    e.pos.0,
                    e.pos.1,
                );
                println!("{} |",space);
                print!("{} |     ",e.pos.0);
                for (i,ch) in trimed_snippet.chars().enumerate(){
                    if i == error_col{
                        print!("{}",ch.to_string().red());
                    }
                    else {
                        print!("{}",ch.to_string());
                    }
                }
                let mut arrow = std::iter::repeat(" ").take(error_col).collect::<String>();
                arrow.push_str("^");
                println!("\n{} |     {}",space,arrow.red());
            }
            println!("{:#?}",symbol_table);
        }
    }
    

    // let mut tkn = tokenizer::Tokenizer::new(source_code.as_str());
    // while !tkn.is_finished() {
    //     println!("{:#?}",tkn.get_next_token());
    // }
    // bin_location.pop();
    // //let mut source_name = args.source.file_stem().unwrap();
    // bin_location.push([String::from("temp_") , args.source.file_stem().unwrap().to_str().unwrap().to_string(),String::from(".c")].concat());
    
    // println!("{:?}",bin_location);
    // let mut file = fs::File::create(bin_location).unwrap();
    // let _ = file.write(gen_c_program(&parser.parse()).as_bytes());
    
}
