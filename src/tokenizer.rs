use std::{mem::discriminant, rc::Rc, vec};
use enum_map::Enum;
use fancy_regex::Regex;

use crate::ErrorPipeline;


#[derive(Debug,PartialEq,Clone,Copy)]
pub enum KeyWords{
    FUNC,
    IMPORT,
    AS,
    CONST,
    FOR,
    IN,
    WHILE,
    IF,
    ELSE,
    ELSEIF,
    RETURN,
    LET
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
    Char,
    Boolean,
    Str(u32),
    Array(Array)
}

#[derive(Debug,PartialEq,Clone)]
pub struct Array {
    pub length:u32,
    pub data_type:Box<DataType>
}

impl DataType{
    pub fn to_string(&self) -> String{
        match self{
            DataType::Void => "void".to_string(),
            DataType::I8 => "i8".to_string(),
            DataType::I16 => "i16".to_string(),
            DataType::I32 => "i32".to_string(),
            DataType::I64 => "i64".to_string(),
            DataType::U8 => "u8".to_string(),
            DataType::U16 => "u16".to_string(),
            DataType::U32 => "u32".to_string(),
            DataType::U64 => "u64".to_string(),
            DataType::F32 => "f32".to_string(),
            DataType::F64 => "f64".to_string(),
            DataType::Char => "char".to_string(),
            DataType::Boolean => "bool".to_string(),
            DataType::Str(_) => "str".to_string(),
            DataType::Array(arr) => format!("array [{}]",arr.data_type.to_string())
        }
    }
    // pub fn to_c_type_string(&self) -> String{
    //     match self{
    //         DataType::Void => "void",
    //         DataType::I8 => "int8_t",
    //         DataType::I16 => "int16_t",
    //         DataType::I32 => "int32_t",
    //         DataType::I64 => "int64_t",
    //         DataType::U8 => "uint8_t",
    //         DataType::U16 => "uint16_t",
    //         DataType::U32 => "uint32_t",
    //         DataType::U64 => "uint64_t",
    //         DataType::F32 => "float",
    //         DataType::F64 => "double",
    //         DataType::Char => "char",
    //         DataType::Boolean => "bool",
    //         DataType::Array(_) => "[]",
    //     }.to_string()
    // }
    pub fn get_size_in_bytes(&self) -> u32{
        match self{
            DataType::Void => 0,
            DataType::I8 => 2,
            DataType::I16 => 4,
            DataType::I32 => 8,
            DataType::I64 => 16,
            DataType::U8 => 2,
            DataType::U16 => 4,
            DataType::U32 => 8,
            DataType::U64 => 16,
            DataType::F32 => 8,
            DataType::F64 => 16,
            DataType::Char => 4,
            DataType::Boolean => 1,
            DataType::Str(length) => 4,
            DataType::Array(arr) => {
                arr.length * arr.data_type.get_size_in_bytes()
                // while discriminant(arr) == discriminant(DataType::Array(Array { length: 0, data_type: Void })){

                // }
            }
        }
    }
}

// impl DataType {
//     pub fn clone(&self) -> DataType {
//         match self{
//             DataType::Void => {DataType::Void},
//             DataType::I8 => {DataType::I8},
//             DataType::I16 => {DataType::I16},
//             DataType::I32 => {DataType::I32},
//             DataType::I64 => {DataType::I64},
//             DataType::U8 => {DataType::U8},
//             DataType::U16 => {DataType::U16},
//             DataType::U32 => {DataType::U32},
//             DataType::U64 => {DataType::U64},
//             DataType::F32 => {DataType::F32},
//             DataType::F64 => {DataType::F64},
//             DataType::String => {DataType::String},
//             DataType::Boolean => {DataType::Boolean},
//         }
//     }
// }

#[derive(Debug,PartialEq,Clone)]
pub enum TokenType{
    Keyword(KeyWords),
    DataType(DataType),
    Identifier(String),
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    CharLiteral(char),
    BooleanLiteral(bool),
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
    And,
    Or,
    Not,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Range,
    Dot,
    SingleQuote,
    DoubleQuote,
    ScopeResolution,
    Colon,
    SemiColon,
    EOF
}

#[derive(Debug,PartialEq,Clone)]
pub struct Token {
    pub token:TokenType,
    pub length:u32,
    pub pos:(u32,u32)
}

// pub enum GroupToken{
//     LeftParen,
//     RightParen,
//     LeftBrace,
//     RightBrace,
// }

// pub enum LiteralToken{
//     Identifier(String),
//     IntLiteral(i32),
//     FloatLiteral(f64),
//     StringLiteral(String),
// }
// pub enum ComparisonToken{
//     Equal,
//     Less,
//     LessEqual,
//     More,
//     MoreEqual,
// }

// pub enum OperatorToken{
//     AdditionOperator,
//     SubtractionOperator,
//     MultiplicationOperator,
//     DivisionOperator,
//     ModuloOperator,
//     AssignmentOperator,
// }

pub struct Tokenizer<'a,'b>{
    pos:(u32,u32),
    cursor:u32,
    source:&'a str,
    rules:Vec<(Regex,TokenType)>,
    error_pipe:&'b ErrorPipeline
}

impl<'a,'b> Tokenizer<'a,'b>{
    pub fn new(source_code:&'a str,error_pipe:&'b ErrorPipeline) -> Tokenizer<'a,'b> {
        return Tokenizer{
            pos:(1,1),
            cursor:0,
            source:source_code,
            error_pipe,
            rules:vec![
                (Regex::new(r"\Afunc\s+").unwrap(),TokenType::Keyword(KeyWords::FUNC)),
                (Regex::new(r"\Aimport\s+").unwrap(),TokenType::Keyword(KeyWords::IMPORT)),
                (Regex::new(r"\Aas\s+").unwrap(),TokenType::Keyword(KeyWords::AS)),
                (Regex::new(r"\Aconst\s+").unwrap(),TokenType::Keyword(KeyWords::CONST)),
                (Regex::new(r"\Areturn(?=\s+|\(|;)").unwrap(),TokenType::Keyword(KeyWords::RETURN)),
                (Regex::new(r"\Alet(?=\s+|\()").unwrap(),TokenType::Keyword(KeyWords::LET)),
                (Regex::new(r"\Awhile(?=\s+|\()").unwrap(),TokenType::Keyword(KeyWords::WHILE)),
                (Regex::new(r"\Afor(?=\s+|\()").unwrap(),TokenType::Keyword(KeyWords::FOR)),
                (Regex::new(r"\Aif(?=\s+|\()").unwrap(),TokenType::Keyword(KeyWords::IF)),
                (Regex::new(r"\Aelse(?=\s+|\{)").unwrap(),TokenType::Keyword(KeyWords::ELSE)),
                (Regex::new(r"\Aelse\s+if(?=\s+|\()").unwrap(),TokenType::Keyword(KeyWords::ELSEIF)),
                (Regex::new(r"\Ain(?=\s+|\()").unwrap(),TokenType::Keyword(KeyWords::IN)),
                (Regex::new(r#"\A".*""#).unwrap(),TokenType::StringLiteral(String::new())),
                (Regex::new(r#"\A'*'"#).unwrap(),TokenType::CharLiteral('a')),
                (Regex::new(r"\Ai8(?=\W)").unwrap(),TokenType::DataType(DataType::I8)),
                (Regex::new(r"\Ai16(?=\W)").unwrap(),TokenType::DataType(DataType::I16)),
                (Regex::new(r"\Ai32(?=\W)").unwrap(),TokenType::DataType(DataType::I32)),
                (Regex::new(r"\Ai64(?=\W)").unwrap(),TokenType::DataType(DataType::I64)),
                (Regex::new(r"\Au8(?=\W)").unwrap(),TokenType::DataType(DataType::U8)),
                (Regex::new(r"\Au16(?=\W)").unwrap(),TokenType::DataType(DataType::U16)),
                (Regex::new(r"\Au32(?=\W)").unwrap(),TokenType::DataType(DataType::U32)),
                (Regex::new(r"\Au64(?=\W)").unwrap(),TokenType::DataType(DataType::U64)),
                (Regex::new(r"\Af32(?=\W)").unwrap(),TokenType::DataType(DataType::F32)),
                (Regex::new(r"\Af64(?=\W)").unwrap(),TokenType::DataType(DataType::F64)),
                (Regex::new(r"\Avoid(?=\W)").unwrap(),TokenType::DataType(DataType::Void)),
                (Regex::new(r"\Achar(?=\W)").unwrap(),TokenType::DataType(DataType::Char)),
                (Regex::new(r"\Astr(?=\W)").unwrap(),TokenType::DataType(DataType::Str(0))),
                (Regex::new(r"\Abool(?=\W)").unwrap(),TokenType::DataType(DataType::Boolean)),
                (Regex::new(r"\Atrue(?=\W)").unwrap(),TokenType::BooleanLiteral(true)),
                (Regex::new(r"\Afalse(?=\W)").unwrap(),TokenType::BooleanLiteral(false)),
                (Regex::new(r"\A[a-zA-Z]+[_0-9]*[_a-zA-Z0-9]*").unwrap(),TokenType::Identifier(String::new())),
                (Regex::new(r"\A[0-9]+\.(?!\.)[0-9]*").unwrap(),TokenType::FloatLiteral(0.0)),
                (Regex::new(r"\A[0-9]+").unwrap(),TokenType::IntLiteral(0)),
                (Regex::new(r"\A\+").unwrap(),TokenType::AdditionOperator),
                (Regex::new(r"\A-").unwrap(),TokenType::SubtractionOperator),
                (Regex::new(r"\A\*").unwrap(),TokenType::MultiplicationOperator),
                (Regex::new(r"\A/").unwrap(),TokenType::DivisionOperator),
                (Regex::new(r"\A%").unwrap(),TokenType::ModuloOperator),
                (Regex::new(r"\A==").unwrap(),TokenType::Equal),
                (Regex::new(r"\A=").unwrap(),TokenType::AssignmentOperator),
                (Regex::new(r"\A::").unwrap(),TokenType::ScopeResolution),
                (Regex::new(r"\A:").unwrap(),TokenType::Colon),
                (Regex::new(r"\A>=").unwrap(),TokenType::MoreEqual),
                (Regex::new(r"\A>").unwrap(),TokenType::More),
                (Regex::new(r"\A<=").unwrap(),TokenType::LessEqual),
                (Regex::new(r"\A<").unwrap(),TokenType::Less),
                (Regex::new(r"\A\&\&").unwrap(),TokenType::And),
                (Regex::new(r"\A\|\|").unwrap(),TokenType::Or),
                (Regex::new(r"\A!").unwrap(),TokenType::Not),
                (Regex::new(r"\A'").unwrap(),TokenType::DoubleQuote),
                (Regex::new(r#"\A""#).unwrap(),TokenType::DoubleQuote),
                (Regex::new(r"\A\(").unwrap(),TokenType::LeftParen),
                (Regex::new(r"\A\)").unwrap(),TokenType::RightParen),
                (Regex::new(r"\A\{").unwrap(),TokenType::LeftBrace),
                (Regex::new(r"\A}").unwrap(),TokenType::RightBrace),
                (Regex::new(r"\A\[").unwrap(),TokenType::LeftBracket),
                (Regex::new(r"\A\]").unwrap(),TokenType::RightBracket),
                (Regex::new(r"\A,").unwrap(),TokenType::Comma),
                (Regex::new(r"\A\.\.").unwrap(),TokenType::Range),
                (Regex::new(r"\A\.").unwrap(),TokenType::Dot),
                (Regex::new(r"\A;").unwrap(),TokenType::SemiColon),
            ]
        }
    }

    pub fn is_finished(&self) -> bool {
        return self.cursor>=self.source.len() as u32;
    }

    pub fn get_next_token(&mut self) -> Token {

        if self.is_finished() {
            return Token {token:TokenType::EOF,length:0,pos:self.pos};
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
            if current_char == '\n'{
                self.pos.0 += 1;
                self.pos.1 = 1;
            }
            self.cursor += 1;
            self.pos.1 +=1;
            if self.is_finished() {
                return Token {token:TokenType::EOF,length:0,pos:self.pos};
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
                Ok(Some(cap))=>{
                    let pos = self.pos;
                    let length = cap.as_str().len() as u32;
                    let current_t = match token{
                        TokenType::Keyword(keyword) => {
                            TokenType::Keyword(keyword.clone())
                        },
                        TokenType::DataType(data_type) => {
                            TokenType::DataType(data_type.clone())
                        },
                        TokenType::IntLiteral(_) => {
                            TokenType::IntLiteral(cap.as_str().parse().unwrap())
                        },
                        TokenType::FloatLiteral(_) => {
                            TokenType::FloatLiteral(cap.as_str().parse().unwrap())
                        },
                        TokenType::Identifier(_) => {
                            TokenType::Identifier(cap.as_str().to_string())
                        },
                        TokenType::StringLiteral(_) => {
                            TokenType::StringLiteral(cap.as_str()[1..cap.as_str().len()-1].to_string())
                        },
                        TokenType::CharLiteral(_) => {
                            if cap.as_str().len() > 3 {
                                self.error_pipe.raise_error(crate::ErrorType::LexicalError, "char literal contain multiple character, consider using str instead", self.pos, 1);
                            }
                            TokenType::CharLiteral(cap.as_str().chars().nth(1).unwrap())
                        },
                        TokenType::AdditionOperator => {
                            TokenType::AdditionOperator
                        },
                        TokenType::SubtractionOperator => {
                            TokenType::SubtractionOperator
                        },
                        TokenType::MultiplicationOperator => {
                            TokenType::MultiplicationOperator
                        },
                        TokenType::DivisionOperator => {
                            TokenType::DivisionOperator
                        },
                        TokenType::ModuloOperator => {
                            TokenType::ModuloOperator
                        },
                        TokenType::AssignmentOperator => {
                            TokenType::AssignmentOperator
                        },
                        TokenType::ScopeResolution => {
                            TokenType::ScopeResolution
                        },
                        TokenType::Colon => {
                            TokenType::Colon
                        },
                        TokenType::Equal => {
                            TokenType::Equal
                        },
                        TokenType::Less => {
                            TokenType::Less
                        },
                        TokenType::LessEqual => {
                            TokenType::LessEqual
                        },
                        TokenType::More => {
                            TokenType::More
                        },
                        TokenType::MoreEqual => {
                            TokenType::MoreEqual
                        },
                        TokenType::LeftParen => {
                            TokenType::LeftParen
                        },
                        TokenType::RightParen => {
                            TokenType::RightParen
                        },
                        TokenType::LeftBrace => {
                            TokenType::LeftBrace
                        },
                        TokenType::RightBrace => {
                            TokenType::RightBrace
                        },
                        TokenType::Comma => {
                            TokenType::Comma
                        },
                        TokenType::Range => {
                            TokenType::Range
                        },
                        TokenType::Dot => {
                            TokenType::Dot
                        },
                        TokenType::SemiColon => {
                            TokenType::SemiColon
                        },
                        TokenType::EOF => {
                            TokenType::EOF
                        },
                        TokenType::And => {
                            TokenType::And
                        },
                        TokenType::Or => {
                            TokenType::Or
                        },
                        TokenType::Not => {
                            TokenType::Not
                        },
                        TokenType::SingleQuote => {
                            self.error_pipe.raise_error(crate::ErrorType::LexicalError, "unclosed single quote", self.pos, 1);
                            self.cursor = self.source.len() as u32;
                            self.pos.1 += 1;
                            return Token {token:TokenType::EOF,length:0,pos};
                        },
                        TokenType::DoubleQuote => {
                            self.error_pipe.raise_error(crate::ErrorType::LexicalError, "unclosed double quote", self.pos, 1);
                            self.cursor = self.source.len() as u32;
                            self.pos.1 += 1;
                            return Token {token:TokenType::EOF,length:0,pos};
                        },
                        TokenType::LeftBracket => {
                            TokenType::LeftBracket
                        },
                        TokenType::RightBracket => {
                            TokenType::RightBracket
                        },
                        TokenType::BooleanLiteral(b) => {
                            TokenType::BooleanLiteral(*b)
                        },
                    };
                    self.cursor += length;
                    self.pos.1 += length;
                    //println!("{:#?}",current_t);
                    return Token {token:current_t,length,pos};
                },
                Ok(None)=>{
                    continue;
                },
                Err(err)=>{
                    panic!("{:?}",err);
                }
            }
        }
        self.error_pipe.raise_error(crate::ErrorType::LexicalError, "unidentified token", self.pos, 1);
        self.cursor += 1;
        self.pos.1 += 1;
        self.get_next_token()
    }
}
