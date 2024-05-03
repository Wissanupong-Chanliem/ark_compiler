use std::vec;
use regex::Regex;


#[derive(Debug,PartialEq,Clone,Copy)]
pub enum KeyWords{
    FUNC,
    IMPORT,
    AS,
    CONST,
    FOR,
    WHILE,
    IF,
    ELSE,
    ELSEIF,
    RETURN
}

#[derive(Debug,PartialEq,Clone)]
pub enum DataType{
    Void,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    String,
    Boolean,
}


#[derive(Debug,PartialEq,Clone)]
pub enum Token{
    Keyword(KeyWords),
    DataType(DataType),
    Identifier(String),
    IntLiteral(i32),
    FloatLiteral(f64),
    StringLiteral(String),
    AdditionOperator,
    SubtractionOperator,
    MultiplicationOperator,
    DivisionOperator,
    ModuloOperator,
    AssignmentOperator,
    Equal,
    Less,
    LessEqual,
    More,
    MoreEqual,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    ScopeResolution,
    Colon,
    SemiColon,
    EOF
}

pub enum GroupToken{
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
}

pub enum LiteralToken{
    Identifier(String),
    IntLiteral(i32),
    FloatLiteral(f64),
    StringLiteral(String),
}
pub enum ComparisonToken{
    Equal,
    Less,
    LessEqual,
    More,
    MoreEqual,
}

pub enum OperatorToken{
    AdditionOperator,
    SubtractionOperator,
    MultiplicationOperator,
    DivisionOperator,
    ModuloOperator,
    AssignmentOperator,
}

pub struct Tokenizer<'a>{
    cursor:u32,
    source:&'a str,
    rules:Vec<(Regex,Token)>
}

impl<'a> Tokenizer<'a>{
    pub fn new(source_code:&'a str) -> Tokenizer<'a>{
        return Tokenizer{
            cursor:0,
            source:source_code,
            rules:vec![
                (Regex::new(r"\Afunc\s").unwrap(),Token::Keyword(KeyWords::FUNC)),
                (Regex::new(r"\Aimport\s").unwrap(),Token::Keyword(KeyWords::IMPORT)),
                (Regex::new(r"\Aas\s").unwrap(),Token::Keyword(KeyWords::AS)),
                (Regex::new(r"\Aconst\s").unwrap(),Token::Keyword(KeyWords::CONST)),
                (Regex::new(r"\Areturn\s").unwrap(),Token::Keyword(KeyWords::RETURN)),
                (Regex::new(r"\Awhile\s").unwrap(),Token::Keyword(KeyWords::WHILE)),
                (Regex::new(r"\Afor\s").unwrap(),Token::Keyword(KeyWords::FOR)),
                (Regex::new(r"\Aif[\s(]").unwrap(),Token::Keyword(KeyWords::IF)),
                (Regex::new(r"\Aelse[\s(]").unwrap(),Token::Keyword(KeyWords::ELSE)),
                (Regex::new(r"\Aelse\s+if[\s(]").unwrap(),Token::Keyword(KeyWords::ELSEIF)),
                (Regex::new(r"\Ai8\W").unwrap(),Token::DataType(DataType::I8)),
                (Regex::new(r"\Ai16\W").unwrap(),Token::DataType(DataType::I16)),
                (Regex::new(r"\Ai32\W").unwrap(),Token::DataType(DataType::I32)),
                (Regex::new(r"\Ai64\W").unwrap(),Token::DataType(DataType::I64)),
                (Regex::new(r"\Au8\W").unwrap(),Token::DataType(DataType::U8)),
                (Regex::new(r"\Au16\W").unwrap(),Token::DataType(DataType::U16)),
                (Regex::new(r"\Au32\W").unwrap(),Token::DataType(DataType::U32)),
                (Regex::new(r"\Au64\W").unwrap(),Token::DataType(DataType::U64)),
                (Regex::new(r"\Af32\W").unwrap(),Token::DataType(DataType::F32)),
                (Regex::new(r"\Af64\W").unwrap(),Token::DataType(DataType::F64)),
                (Regex::new(r"\Avoid\W").unwrap(),Token::DataType(DataType::Void)),
                (Regex::new(r"\Astring\W").unwrap(),Token::DataType(DataType::String)),
                (Regex::new(r"\Abool\W").unwrap(),Token::DataType(DataType::Boolean)),
                (Regex::new(r"\A[a-zA-Z]+[_0-9]*[_a-zA-Z0-9]*").unwrap(),Token::Identifier(String::new())),
                (Regex::new(r"\A[0-9]+\.[0-9]*").unwrap(),Token::FloatLiteral(0.0)),
                (Regex::new(r"\A[0-9]+[.]?").unwrap(),Token::IntLiteral(0)),
                (Regex::new(r"\A\+").unwrap(),Token::AdditionOperator),
                (Regex::new(r"\A-").unwrap(),Token::SubtractionOperator),
                (Regex::new(r"\A\*").unwrap(),Token::MultiplicationOperator),
                (Regex::new(r"\A/").unwrap(),Token::DivisionOperator),
                (Regex::new(r"\A%").unwrap(),Token::ModuloOperator),
                (Regex::new(r"\A=").unwrap(),Token::AssignmentOperator),
                (Regex::new(r"\A==").unwrap(),Token::Equal),
                (Regex::new(r"\A::").unwrap(),Token::ScopeResolution),
                (Regex::new(r"\A:").unwrap(),Token::Colon),
                (Regex::new(r"\A>").unwrap(),Token::More),
                (Regex::new(r"\A>=").unwrap(),Token::MoreEqual),
                (Regex::new(r"\A<").unwrap(),Token::Less),
                (Regex::new(r"\A<=").unwrap(),Token::LessEqual),
                (Regex::new(r"\A\(").unwrap(),Token::LeftParen),
                (Regex::new(r"\A\)").unwrap(),Token::RightParen),
                (Regex::new(r"\A\{").unwrap(),Token::LeftBrace),
                (Regex::new(r"\A}").unwrap(),Token::RightBrace),
                (Regex::new(r"\A,").unwrap(),Token::Comma),
                (Regex::new(r"\A\.").unwrap(),Token::Dot),
                (Regex::new(r"\A;").unwrap(),Token::SemiColon),
            ]
        }
    }
    pub fn is_finished(&self) -> bool {
        return self.cursor>=self.source.len() as u32;
    }
    pub fn get_next_token(&mut self) -> Token {
        if self.is_finished() {
            return Token::EOF;
        }
        let get_current_char = self.source.chars().nth(self.cursor as usize);
        let mut current_char = match get_current_char{
            Some(ch) => {
                ch
            },
            None =>{
                panic!("cursor somehow exceed string length");
            }
        };
        while current_char.is_whitespace() {
            self.cursor += 1;
            if self.is_finished() {
                return Token::EOF;
            }
            let get_current_char = self.source.chars().nth(self.cursor as usize);
            current_char = match get_current_char{
                Some(ch) => {
                    ch
                },
                None =>{
                    panic!("cursor somehow exceeded string length.");
                }
            };
        }
        let current_sli = self.source.get((self.cursor as usize)..).unwrap();
        for (rule , token) in &self.rules{
            let match_result = rule.find(current_sli);
            match match_result {
                Some(cap)=>{
                    self.cursor += cap.as_str().len() as u32;
                    match token{
                        Token::Keyword(keyword) => {
                            return Token::Keyword(keyword.clone());
                        },
                        Token::DataType(data_type) => {
                            return Token::DataType(data_type.clone());
                        },
                        Token::IntLiteral(_) => {
                            return Token::IntLiteral(cap.as_str().parse().unwrap());
                        },
                        Token::FloatLiteral(_) => {
                            return Token::FloatLiteral(cap.as_str().parse().unwrap());
                        },
                        Token::Identifier(_) => {
                            return Token::Identifier(cap.as_str().to_string());
                        },
                        Token::StringLiteral(_) => {
                            return Token::StringLiteral(cap.as_str().to_string());
                        },
                        Token::AdditionOperator => {
                            return Token::AdditionOperator;
                        },
                        Token::SubtractionOperator => {
                            return Token::SubtractionOperator;
                        },
                        Token::MultiplicationOperator => {
                            return Token::MultiplicationOperator;
                        },
                        Token::DivisionOperator => {
                            return Token::DivisionOperator;
                        },
                        Token::ModuloOperator => {
                            return Token::ModuloOperator;
                        },
                        Token::AssignmentOperator => {
                            return Token::AssignmentOperator;
                        },
                        Token::ScopeResolution => {
                            return Token::ScopeResolution;
                        },
                        Token::Colon => {
                            return Token::Colon;
                        },
                        Token::Equal => {
                            return Token::Equal;
                        },
                        Token::Less => {
                            return Token::Less;
                        },
                        Token::LessEqual => {
                            return Token::LessEqual;
                        },
                        Token::More => {
                            return Token::More;
                        },
                        Token::MoreEqual => {
                            return Token::MoreEqual;
                        },
                        Token::LeftParen => {
                            return Token::LeftParen;
                        },
                        Token::RightParen => {
                            return Token::RightParen;
                        },
                        Token::LeftBrace => {
                            return Token::LeftBrace;
                        },
                        Token::RightBrace => {
                            return Token::RightBrace;
                        },
                        Token::Comma => {
                            return Token::Comma;
                        },
                        Token::Dot => {
                            return Token::Dot;
                        },
                        Token::SemiColon => {
                            return Token::SemiColon;
                        },
                        Token::EOF => {
                            return Token::EOF;
                        },
                        
                        
                        
                    }
                },
                None=>{
                    continue;
                }
            }
        }
        
        panic!("Unidentified token");
    }
}
