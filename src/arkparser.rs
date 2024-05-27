use crate::{symbol_table::{self, SymbolTable}, tokenizer::{self, Array, DataType, KeyWords, Token, TokenType, Tokenizer}, ErrorPipeline, ErrorType};
use core::fmt;
use std::{mem::{self, discriminant}, path::Display};

#[derive(Debug,Clone)]
pub enum Node {
    Body(Body),
    Variable(String),
    DeclareVar(Var),
    Assignment(BinExp),
    Literal(LiteralValue),
    BinaryExpression(BinExp),
    Function(FuncDef),
    FunctionCall(FuncCall),
    MethodCall(MethodCall),
    Import(Import),
    Return(Option<Box<AstNode<Node>>>),
    Conditional(ConditionalBlock),
    For(ForLoop),
    While(WhileLoop),
    BooleanNot(NotExp),
    Tuple(TupleBody),
    Range(Range),
    ParserError(ParserError),
}

#[derive(Clone)]
pub struct AstNode <T> {
    pub node: T,
    pub pos: (u32,u32),
    pub length: u32,
}

impl<T> AstNode<T>{
    pub fn new(node:T,pos:(u32,u32),length: u32) -> AstNode<T>{
        AstNode { node,pos, length }
    }
}

#[derive(Debug,Clone)]
pub struct ParserError {
    pub error_type:ErrorType,
    pub error_message:String,
    pub pos:(u32,u32)
}


// #[derive(Debug,Clone)]
// pub enum ErrorType {
//     SemanticError,
//     SyntaxError
// }

// impl ErrorType{
//     pub fn as_str(&self) -> &str{
//         match self {
//             ErrorType::SemanticError => "Sematic Error",
//             ErrorType::SyntaxError => "Syntax Error",
//         }
//     }
// }

impl ParserError{
    pub fn new(error_type:ErrorType,error_message:&str,error_pos:(u32,u32)) -> ParserError{
        ParserError {
            error_type,
            error_message:String::from(error_message),
            pos:error_pos,
        }
    }
}

#[derive(Debug,Clone)]
pub enum LiteralValue {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool)
}


#[derive(Debug,Clone)]
pub struct Body {
    pub instructions: Vec<AstNode<Node>>,
}

impl Body {
    pub fn contains(&self, node:&Node) -> bool {
        for instruction in &self.instructions {
            if let Node::Body(bd) = &instruction.node {
                if bd.contains(node){
                    return true;
                }
            }
            else{
                if discriminant(&instruction.node) == discriminant(node){
                    return true;
                }
            }
        }
        return false;
    }
}

impl fmt::Debug for AstNode<Node>{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{:#?}",self.node)
    }
}

impl fmt::Debug for AstNode<DataType>{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{:#?}",self.node)
    }
}

impl fmt::Debug for AstNode<TokenType>{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{:#?}",self.node)
    }
}

impl fmt::Debug for AstNode<String>{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}",self.node)
    }
}

impl fmt::Debug for AstNode<bool>{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}",self.node)
    }
}

impl fmt::Debug for AstNode<Var>{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{:#?}",self.node)
    }
}


#[derive(Debug,Clone)]
pub struct ConditionalBlock{
    pub if_block:(Box<AstNode<Node>>,Body),
    pub elif_block:Vec<(AstNode<Node>,Body)>,
    pub else_block:Option<Body>,
}

#[derive(Debug,Clone)]
pub struct Range{
    pub start:Box<AstNode<Node>>,
    pub end:Box<AstNode<Node>>,
}

#[derive(Debug,Clone)]
pub struct ForLoop{
    pub var:Option<Box<AstNode<Node>>>,
    pub range:Box<AstNode<Node>>,
    pub body:Body,
}

#[derive(Debug,Clone)]
pub struct WhileLoop{
    pub condition:Box<AstNode<Node>>,
    pub body:Body,
}

#[derive(Debug,Clone)]
pub struct TupleBody {
    pub members: Vec<AstNode<Node>>,
}

#[derive(Debug,Clone)]
pub struct BinExp {
    pub left: Box<AstNode<Node>>,
    pub right: Box<AstNode<Node>>,
    pub operator: AstNode<TokenType>,
    //pub pos:(u32,u32)
}


#[derive(Debug,Clone)]
pub struct NotExp {
    pub exp: Box<AstNode<Node>>
}

#[derive(Debug,Clone)]
pub struct Var {
    pub constant:Option<AstNode<bool>>,
    pub name: AstNode<String>,
    pub var_type: AstNode<DataType>,
}

#[derive(Debug,Clone)]
pub struct FuncDef {
    pub function_name: AstNode<String>,
    pub body:Body,
    pub return_type: AstNode<DataType>,
    pub parameters: Vec<AstNode<Var>>,
}

#[derive(Debug,Clone)]
pub struct FuncCall {
    pub function_name: AstNode<String>,
    pub arguments: Vec<AstNode<Node>>,
}

#[derive(Debug,Clone)]
pub struct MethodCall {
    pub caller:Option<Box<AstNode<Node>>>,
    pub method_name: AstNode<String>,
    pub arguments: Vec<AstNode<Node>>,
}


#[derive(Debug,Clone)]
pub struct Import {
    pub import_name: AstNode<String>,
    pub alias: Option<AstNode<String>>,
}

pub struct ArkParser<'source,'error_pipe,'tokenizer> {
    tokenizer: &'tokenizer mut Tokenizer<'source,'error_pipe>,
    look_ahead: Token,
    err_pipe:&'error_pipe ErrorPipeline,
}

impl<'source,'error_pipe,'tokenizer> ArkParser<'source,'error_pipe,'tokenizer> {
    pub fn new(tokenizer:&'tokenizer mut Tokenizer<'source,'error_pipe>,error_pipe: &'error_pipe ErrorPipeline) -> ArkParser<'source,'error_pipe,'tokenizer> {
        return ArkParser {
            look_ahead: tokenizer.get_next_token(),
            tokenizer,
            err_pipe:error_pipe,
        };
    }

    fn expected(&mut self, expected: &TokenType) -> bool {
        match expected{
            TokenType::Keyword(keyword) =>{
                if mem::discriminant(&self.look_ahead.token) == mem::discriminant(expected) {
                    match &self.look_ahead.token{
                        TokenType::Keyword(look_ahead_key) => {
                            return mem::discriminant(look_ahead_key) == mem::discriminant(&keyword);
                        },
                        _ => {
                            return false;
                        },
                    }
                    
                }
                return false;
            },
            _ => {
                return mem::discriminant(&self.look_ahead.token) == mem::discriminant(expected);
            }
        }
        //return self.look_ahead.token == expected;
    }

    fn eat(&mut self, expected: &TokenType,error_message:&str) -> Result<AstNode<TokenType>,AstNode<Node>> {
        let pos = self.look_ahead.pos;
        let length = self.look_ahead.length;
        
        match expected{
            TokenType::Keyword(keyword) =>{
                let node = self.look_ahead.token.clone();
                if mem::discriminant(&self.look_ahead.token) == mem::discriminant(&expected) {
                    match &self.look_ahead.token{
                        TokenType::Keyword(look_ahead_key) => {
                            if mem::discriminant(look_ahead_key) == mem::discriminant(keyword) {
                                self.look_ahead = self.tokenizer.get_next_token();
                                return Ok(AstNode::new(node, pos, length));
                            }
                            // else{
                            //     self.raise_error(error_message);
                            //     self.look_ahead = self.tokenizer.get_next_token();
                            //     return Ok(AstNode::new(expected.clone(), pos, length));
                            // }
                            
                        },
                        _ => {
                            // self.raise_error(error_message);
                            // self.look_ahead = self.tokenizer.get_next_token();
                            // return Ok(AstNode::new(expected.clone(), pos, length));
                        },
                    }
                    
                }
            },
            _ => {
                let node = self.look_ahead.token.clone();
                if mem::discriminant(&node) == mem::discriminant(expected) {
                    self.look_ahead = self.tokenizer.get_next_token();
                    return Ok(AstNode::new(node, pos, length));
                }
                // else{
                //     self.raise_error( error_message);
                //     self.look_ahead = self.tokenizer.get_next_token();
                //     return Ok(AstNode::new(expected.clone(), pos, length));
                // }
            }
        }
        //println!("expected: {:#?} founded: {:#?}",expected,self.look_ahead);
        //self.look_ahead = self.tokenizer.get_next_token();
        //self.skip_until_delim();
        return Err(
            AstNode { node: Node::ParserError(self.raise_error(error_message)), pos, length }
        );
        
    }
    fn skip_until(&mut self,delimiter:&TokenType){
        
        while !self.expected(delimiter) && !self.expected(&TokenType::EOF) {
            //println!("current: {:#?}",self.look_ahead);
            self.look_ahead = self.tokenizer.get_next_token();
        }
    }
    fn skip_block(&mut self){
        let mut s: Vec<bool> = vec![];
        
        if self.expected(&TokenType::LeftBrace){
            s.push(true);
            let open = self.eat(&TokenType::LeftBrace, "expected open brace").unwrap();
            while !s.is_empty(){
                //println!("skip: {:#?}",self.look_ahead);
                if self.expected(&TokenType::EOF){
                    self.err_pipe.raise_error(
                        ErrorType::SyntaxError,
                        "unclosed brace",
                        self.look_ahead.pos,
                        self.look_ahead.length
                    );
                    return;
                }
                else if self.expected(&TokenType::LeftBrace){
                    s.push(true);
                }
                else if self.expected(&TokenType::RightBrace){
                    s.pop();
                }
                self.look_ahead = self.tokenizer.get_next_token();
            }
            //self.eat(&TokenType::RightBrace, "expected open brace").unwrap();
        }
    }
    fn raise_error(&mut self,error_message:&str) -> ParserError {
        
        let error_pos = self.look_ahead.pos;
        self.err_pipe.raise_error(crate::ErrorType::SyntaxError, error_message, error_pos, 1);

        let err = ParserError::new(
            ErrorType::SyntaxError,
            error_message,
            error_pos
        );
        //self.syntax_errors.push(err.clone());
        //self.look_ahead = self.tokenizer.get_next_token();
        err
    }

    fn parse_data_type(&mut self) -> Result<AstNode<DataType>,AstNode<Node>> {
        let data_type_t = match self.eat(&TokenType::DataType(DataType::Void), "Expected a type"){
            Ok(dt) => {dt},
            Err(e) => return Err(e)
        };
        let mut data_type = if let TokenType::DataType(t) = data_type_t.node {
            t
        }
        else{
            DataType::Void
        };
        //let mut array:Array = vec![];
        while self.expected(&TokenType::LeftBracket){
            self.eat(&TokenType::LeftBracket, "");
            let size = match self.eat(&TokenType::IntLiteral(0), "expected fixed array size"){
                Ok(node) => if let TokenType::IntLiteral(i) = node.node{
                    i
                }
                else {
                    0
                },
                Err(e) => return Err(e)
            };
            match self.eat(&TokenType::RightBracket, "expected closing bracket"){
                Ok(_) => (),
                Err(e) => return Err(e)
            };
            data_type = DataType::Array(Array{
                length: size as u32,
                data_type: Box::from(data_type),
            });
        }
        Ok(
            AstNode {
                node: data_type,
                pos: data_type_t.pos,
                length: data_type_t.length
            }
        )
    }
    fn parse_block(&mut self) -> Body {
        
        let mut scope_body = Body {
            instructions: vec![],
        };
        let l_brace = self.eat(&TokenType::LeftBrace, "expected a block").unwrap();
        while !self.expected(&TokenType::RightBrace) {
            
            let node:AstNode<Node> = match self.look_ahead.token {
                
                TokenType::Keyword(keyword) => {
                    
                    match keyword {
                        KeyWords::FUNC => {
                            let er = self.raise_error(
                                "only top-level function declaration is allowed",
                            );
                            AstNode::new(
                                Node::ParserError(
                                    er.clone()
                                ),
                                er.pos,
                                0
                            )
                            // let func = self.parse_function();
                            // println!("{:#?}",&func);
                            // func
                        },
                        KeyWords::IMPORT => {
                            
                            let mut res = self.parse_import();
                            match self.eat(&TokenType::SemiColon,"missing semicolon") {
                                Ok(_)=>(),
                                Err(e) => {
                                    res = e;
                                }
                            };
                            res

                        },
                        KeyWords::AS => AstNode {
                            node: Node::ParserError(
                                self.raise_error(
                                    "expected token 'import' before 'sources' token"
                                )
                            ),
                            length:self.look_ahead.length,
                            pos:self.look_ahead.pos
                        },
                        KeyWords::CONST => {
                            let mut res = self.parse_iden_init(true);
                            match self.eat(&TokenType::SemiColon,"missing semicolon") {
                                Ok(_)=>(),
                                Err(e) => {
                                    res = e;
                                }
                            };
                            res
                        },
                        KeyWords::LET => {
                            let mut res = self.parse_iden_init(false);
                            match self.eat(&TokenType::SemiColon,"missing semicolon") {
                                Ok(_)=>(),
                                Err(e) => {
                                    res = e;
                                }
                            };
                            res
                        },
                        KeyWords::RETURN => {
                            let mut res = self.parse_return();
                            match self.eat(&TokenType::SemiColon,"missing semicolon") {
                                Ok(_)=>(),
                                Err(e) => {
                                    res = e;
                                }
                            };
                            res
                        },
                        KeyWords::FOR => {
                            self.parse_for()
                        },
                        KeyWords::WHILE => self.parse_while(),
                        KeyWords::IF => self.parse_condition(),
                        KeyWords::ELSEIF => {
                            let er = self.raise_error(
                                "expected token 'if' before 'else if' token",
                            );
                            AstNode::new(
                                Node::ParserError(
                                    er.clone()
                                ),
                                er.pos,
                                0
                            )
                        },
                        KeyWords::ELSE => {
                            let er = self.raise_error(
                                "expected token 'if' before 'else' token",
                            );
                            AstNode::new(
                                Node::ParserError(
                                    er.clone()
                                ),
                                er.pos,
                                0
                            )
                        },
                        KeyWords::IN => {
                            let er = self.raise_error(
                                "expected identifier or tuple of identifier before 'in' token",
                            );
                            AstNode::new(
                                Node::ParserError(
                                    er.clone()
                                ),
                                er.pos,
                                0
                            )
                        },
                    }
                },
                TokenType::Identifier(_) => {
                    
                    let mut res = self.parse_iden();
                    match self.eat(&TokenType::SemiColon,"missing semicolon") {
                        Ok(_)=>(),
                        Err(e) => {
                            res = e;
                        }
                    };
                    res
                },
                TokenType::MultiplicationOperator => {todo!()},
                TokenType::And => {
                    let er = self.raise_error(
                        "expected identifier or expression before '&&' token",
                    );
                    AstNode::new(
                        Node::ParserError(
                            er.clone()
                        ),
                        er.pos,
                        0
                    )
                },
                TokenType::Or => {
                    let er = self.raise_error(
                        "expected identifier or expression before '||' token",
                    );
                    AstNode::new(
                        Node::ParserError(
                            er.clone()
                        ),
                        er.pos,
                        0
                    )
                },
                TokenType::RightParen => {
                    let er = self.raise_error(
                        "unmatched parenthesis",
                    );
                    AstNode::new(
                        Node::ParserError(
                            er.clone()
                        ),
                        er.pos,
                        0
                    )
                },
                TokenType::RightBrace => {
                    let er = self.raise_error(
                        "unmatched brace",
                    );
                    AstNode::new(
                        Node::ParserError(
                            er.clone()
                        ),
                        er.pos,
                        0
                    )
                },
                TokenType::EOF => {
                    //let _er = self.raise_error("unclosed brace");
                    self.err_pipe.raise_error(crate::ErrorType::SyntaxError, "unclosed brace", l_brace.pos, 1);
                    break;
                },
                _ => {
                    //println!("{:#?}",self.look_ahead.token);
                    todo!();
                },
            };
            scope_body.instructions.push(node);
        }
        
        if self.expected(&TokenType::RightBrace) {
            let _ = self.eat(&TokenType::RightBrace, "unclosed brace");
        }
        return scope_body;
    }

    fn parse_body(&mut self,stop_token:TokenType) -> Body {
        let mut scope_body = Body {
            instructions: vec![],
        };
        
        while !self.expected(&stop_token) {
            
            let node:AstNode<Node> = match self.look_ahead.token {
                
                TokenType::Keyword(keyword) => {
                    
                    match keyword {
                        KeyWords::FUNC => {

                            let func = self.parse_function();
                            //println!("{:#?}",&func);
                            func
                        },
                        KeyWords::IMPORT => {
                            
                            let mut res = self.parse_import();
                            match self.eat(&TokenType::SemiColon,"missing semicolon") {
                                Ok(_)=>(),
                                Err(e) => {
                                    res = e;
                                }
                            };
                            res

                        },
                        KeyWords::AS => AstNode {
                            node: Node::ParserError(
                                self.raise_error(
                                    "expected token 'import' before 'sources' token"
                                )
                            ),
                            length:self.look_ahead.length,
                            pos:self.look_ahead.pos
                        },
                        KeyWords::CONST => {
                            let mut res = self.parse_iden_init(true);
                            match self.eat(&TokenType::SemiColon,"missing semicolon") {
                                Ok(_)=>(),
                                Err(e) => {
                                    res = e;
                                }
                            };
                            res
                        },
                        KeyWords::LET => {
                            let mut res = self.parse_iden_init(false);
                            match self.eat(&TokenType::SemiColon,"missing semicolon") {
                                Ok(_)=>(),
                                Err(e) => {
                                    res = e;
                                }
                            };
                            res
                        },
                        KeyWords::RETURN => {
                            let mut res = self.parse_return();
                            match self.eat(&TokenType::SemiColon,"missing semicolon") {
                                Ok(_)=>(),
                                Err(e) => {
                                    res = e;
                                }
                            };
                            res
                        },
                        KeyWords::FOR => {
                            self.parse_for()
                        },
                        KeyWords::WHILE => self.parse_while(),
                        KeyWords::IF => self.parse_condition(),
                        KeyWords::ELSEIF => {
                            let er = self.raise_error(
                                "expected token 'if' before 'else if' token",
                            );
                            AstNode::new(
                                Node::ParserError(
                                    er.clone()
                                ),
                                er.pos,
                                0
                            )
                        },
                        KeyWords::ELSE => {
                            let er = self.raise_error(
                                "expected token 'if' before 'else' token",
                            );
                            AstNode::new(
                                Node::ParserError(
                                    er.clone()
                                ),
                                er.pos,
                                0
                            )
                        },
                        KeyWords::IN => {
                            let er = self.raise_error(
                                "expected identifier or tuple of identifier before 'in' token",
                            );
                            AstNode::new(
                                Node::ParserError(
                                    er.clone()
                                ),
                                er.pos,
                                0
                            )
                        },
                    }
                },
                TokenType::Identifier(_) => {
                    
                    let mut res = self.parse_iden();
                    match self.eat(&TokenType::SemiColon,"missing semicolon") {
                        Ok(_)=>(),
                        Err(e) => {
                            res = e;
                        }
                    };
                    res
                },
                TokenType::MultiplicationOperator => {todo!()},
                TokenType::And => {
                    let er = self.raise_error(
                        "expected identifier or expression before '&&' token",
                    );
                    AstNode::new(
                        Node::ParserError(
                            er.clone()
                        ),
                        er.pos,
                        0
                    )
                },
                TokenType::Or => {
                    let er = self.raise_error(
                        "expected identifier or expression before '||' token",
                    );
                    AstNode::new(
                        Node::ParserError(
                            er.clone()
                        ),
                        er.pos,
                        0
                    )
                },
                TokenType::RightParen => {
                    let er = self.raise_error(
                        "unmatched parenthesis",
                    );
                    AstNode::new(
                        Node::ParserError(
                            er.clone()
                        ),
                        er.pos,
                        0
                    )
                },
                TokenType::RightBrace => {
                    let er = self.raise_error(
                        "unmatched brace",
                    );
                    AstNode::new(
                        Node::ParserError(
                            er.clone()
                        ),
                        er.pos,
                        0
                    )
                },
                _ => {
                    //println!("{:#?}",self.look_ahead.token);
                    todo!();
                },
            };
            scope_body.instructions.push(node);
        }
        return scope_body;
    }
    fn parse_import(&mut self) -> AstNode<Node> {
        let import_keyword = match self.eat(&TokenType::Keyword(KeyWords::IMPORT),"") {
            Ok(t)=>t,
            Err(e) => {return e;}
        };
        let import_name = match self.eat(&TokenType::StringLiteral(String::new()),"expected import name after 'import'") {
            Ok(t)=>t,
            Err(e) => {return e;}
        };
        let name = if let TokenType::StringLiteral(s) = import_name.node {
            s
        } else {
            String::new()
        };
        if self.expected(&TokenType::Keyword(KeyWords::AS)) {
            let _import_as = match self.eat(&TokenType::Keyword(KeyWords::AS),"")  {
                Ok(t)=>t,
                Err(e) => {return e;}
            };
            let import_alias = match self.eat(&TokenType::Identifier(String::new()),"expected identifier after 'sources'") {
                Ok(t)=>t,
                Err(e) => {return e;}
            };
            
            let alias = if let TokenType::Identifier(id) = import_alias.node {
                id
            } else {
                String::new()
            };
            return AstNode {
                node:
                Node::Import(Import {
                    import_name: AstNode { node: name, pos: import_name.pos, length: import_name.length },
                    alias: Some(AstNode { node: alias, pos: import_alias.pos, length: import_alias.length }),
                }),
                pos:import_keyword.pos,
                length:import_keyword.length,
            };
        }
        return AstNode {
            node:
            Node::Import(Import {
                import_name: AstNode { node: name, pos: import_name.pos, length: import_name.length },
                alias: None
            }),
            pos:import_keyword.pos,
            length:import_keyword.length,
        };
    }
    
    fn parse_function(&mut self) -> AstNode<Node> {
        let func_keyword = match self.eat(&TokenType::Keyword(KeyWords::FUNC),"") {
            Ok(t)=>t,
            Err(e) => {return e;}
        };
        let func_name = match self.eat(&TokenType::Identifier(String::new()),"expect a function name") {
            Ok(t)=>t,
            Err(e) => {
                self.skip_until(&TokenType::LeftBrace);
                self.skip_block();
                return e;
            }
        };
        let function_name = if let TokenType::Identifier(id) = func_name.node {
            id
        } else {
            String::new()
        };
        let _ = match self.eat(&TokenType::LeftParen,"expected function parameter") {
            Ok(t)=>t,
            Err(e) => {
                self.skip_until(&TokenType::LeftBrace);
                self.skip_block();
                return e;
            }
        };
        let mut parameters: Vec<AstNode<Var>> = vec![];
        while !self.expected(&TokenType::RightParen){
            
            let para_name_t = match self.eat(&TokenType::Identifier(String::new()),"missing parameter name") {
                Ok(t)=>t,
                Err(e) => {
                    self.skip_until(&TokenType::LeftBrace);
                    
                    self.skip_block();
                    return e;
                }
            };
            let para_name = if let TokenType::Identifier(id) = para_name_t.node {
                id
            }
            else{
                String::new()
            };
            match self.eat(&TokenType::Colon,"expected ':' for type declaration"){
                Ok(t) => t,
                Err(e) => {
                    self.skip_until(&TokenType::LeftBrace);
                    self.skip_block();
                    return e;
                }
            };
            let para_type_t = match self.eat(&TokenType::DataType(DataType::Void),"missing parameter type") {
                Ok(t)=>t,
                Err(e) => {
                    self.skip_until(&TokenType::LeftBrace);
                    self.skip_block();
                    return e;
                }
            };
            let para_type = if let TokenType::DataType(data_type) = para_type_t.node {
                data_type
            }
            else{
                DataType::Void
            };
            parameters.push(
                AstNode::new(
                    Var {
                        constant:None,
                        name:AstNode::new(para_name, para_name_t.pos, para_name_t.length),
                        var_type:AstNode::new(para_type, para_type_t.pos, para_type_t.length)
                    },
                    para_type_t.pos,
                    para_type_t.length + 1 + para_name_t.length
                )
                
            );
            if self.expected(&TokenType::Comma) {
                self.eat(&TokenType::Comma,"");
            }
        }
        let r_paren = match self.eat(&TokenType::RightParen,"") {
            Ok(t)=>t,
            Err(e) => {return e;}
        };
        let mut return_type_token :Option<AstNode<TokenType>> = None;
        let mut return_type = DataType::Void;
        if self.expected(&TokenType::Colon) {
            let _ = self.eat(&TokenType::Colon,"");
            return_type_token = match self.eat(&TokenType::DataType(DataType::Void),"expected function return type, omit ':' if type is void") {
                Ok(t)=> Some(t),
                Err(e) => {return e;}
            };
            return_type = if let TokenType::DataType(data_type) = return_type_token.clone().unwrap().node {
                data_type
            }
            else{
                DataType::Void
            };
        }
        let body = self.parse_block();
        if return_type != DataType::Void && !body.contains(&Node::Return(None)){
            let ret_token = return_type_token.unwrap();
            self.err_pipe.raise_error(ErrorType::SyntaxError, "missing return statement", ret_token.pos, ret_token.length);
        };
        return AstNode::new(
            Node::Function(
                FuncDef {
                    function_name:AstNode::new(
                        function_name,
                        func_name.pos,
                        func_name.length
                    ),
                    body,
                    parameters,
                    return_type:AstNode::new(return_type.clone(),r_paren.pos,return_type.to_string().len() as u32)
                }
            ),
            func_keyword.pos,
            0
        );

    }
    fn parse_return(&mut self) -> AstNode<Node> {
        let ret_kw = match self.eat(&TokenType::Keyword(KeyWords::RETURN),"") {
            Ok(t)=>t,
            Err(e) => {return e;}
        };
        if self.expected(&TokenType::SemiColon) {
            return AstNode::new(Node::Return(None),ret_kw.pos,ret_kw.length+1);
        }
        let exp = self.parse_primary();
        let val = AstNode::new(Node::Return(Some(Box::from(exp.clone()))),ret_kw.pos,ret_kw.length + 1 + exp.length);
        val
    }

    fn parse_condition(&mut self) -> AstNode<Node>{
        let if_condition;
        let if_kw =match self.eat(&TokenType::Keyword(KeyWords::IF),"") {
            Ok(t)=>t,
            Err(e) => {return e;}
        };
        // if self.expected(&TokenType::LeftParen){
        //     self.eat(&TokenType::LeftParen);
        //     if_condition = self.parse_primary();
        //     self.eat(&TokenType::RightParen);
            
        // }
        //else{

        if_condition = self.parse_primary();
        //}
        // match self.eat(&TokenType::LeftBrace) {
        //     Ok(t)=>t,
        //     Err(e) => {return e;}
        // };
        // let if_body = self.parse_body(TokenType::RightBrace);
        // match self.eat(&TokenType::RightBrace) {
        //     Ok(t)=>t,
        //     Err(e) => {return e;}
        // };
        let if_body = self.parse_block();
        let mut elseif_block:Vec<(AstNode<Node>,Body)> = vec![];
        while self.expected(&TokenType::Keyword(KeyWords::ELSEIF)){
            let elif_condition;
            match self.eat(&TokenType::Keyword(KeyWords::ELSEIF),"") {
                Ok(t)=>t,
                Err(e) => {return e;}
            };
            // if self.expected(&TokenType::LeftParen){
            //     self.eat(&TokenType::LeftParen);
            //     elif_condition = self.parse_primary();
            //     self.eat(&TokenType::RightParen);
                
            // }
            // else{
            elif_condition = self.parse_primary();
            //}
            // match self.eat(&TokenType::LeftBrace) {
            //     Ok(t)=>t,
            //     Err(e) => {return e;}
            // };
            // let elif_body = self.parse_body(TokenType::RightBrace);
            // match self.eat(&TokenType::RightBrace) {
            //     Ok(t)=>t,
            //     Err(e) => {return e;}
            // };
            let elif_body = self.parse_block();
            elseif_block.push((elif_condition,elif_body))
        }
        let else_block = if self.expected(&TokenType::Keyword(KeyWords::ELSE)){
            match self.eat(&TokenType::Keyword(KeyWords::ELSE),"") {
                Ok(t)=>t,
                Err(e) => {return e;}
            };
            // match self.eat(&TokenType::LeftBrace) {
            //     Ok(t)=>t,
            //     Err(e) => {return e;}
            // };
            // let else_body = self.parse_body(TokenType::RightBrace);
            // match self.eat(&TokenType::RightBrace) {
            //     Ok(t)=>t,
            //     Err(e) => {return e;}
            // };
            let else_body = self.parse_block();
            Some(else_body)
        }
        else{
            None
        };
        AstNode::new(
            Node::Conditional(
                ConditionalBlock {
                    if_block: (Box::new(if_condition),if_body),
                    elif_block:elseif_block,
                    else_block
                }
            ),
            if_kw.pos,
            0
        )
        
    }
    fn parse_for(&mut self) -> AstNode<Node>{
        let for_kw = match self.eat(&TokenType::Keyword(KeyWords::FOR),"") {
            Ok(t)=>t,
            Err(e) => {return e;}
        };
        let iden = if self.expected(&TokenType::LeftParen) {
            self.parse_paren()
        }
        else {
            let iden = match self.eat(&TokenType::Identifier(String::new()),"expected identifier after 'for'") {
                Ok(t)=>t,
                Err(e) => {return e;}
            };
            let ide= if let TokenType::Identifier(id) = iden.node {
                id
            }
            else{
                String::new()
            };
            AstNode::new(Node::Variable(ide),iden.pos,iden.length)
        };
        match self.eat(&TokenType::Keyword(KeyWords::IN),"expected 'in' keyword") {
            Ok(t)=>t,
            Err(e) => {return e;}
        };
        let range = self.parse_range();

        // match self.eat(&TokenType::LeftBrace,"") {
        //     Ok(t)=>t,
        //     Err(e) => {return e;}
        // };
        let for_body = self.parse_block();//self.parse_body(TokenType::RightBrace);
        // match self.eat(&TokenType::RightBrace) {
        //     Ok(t)=>t,
        //     Err(e) => {return e;}
        // };
        AstNode::new(
            Node::For(ForLoop {
                body:for_body ,
                var:Some(Box::new(iden)),
                range:Box::new(range)
            }),
            for_kw.pos,
            for_kw.length
        )
        
        
    }

    fn parse_range(&mut self) -> AstNode<Node>{
        
        let left = self.parse_primary();
        if self.expected(&TokenType::Range) {
            match self.eat(&TokenType::Range,"expected range") {
                Ok(t)=>t,
                Err(e) => {return e;}
            };
            let right = self.parse_primary();
            return AstNode::new(Node::Range(Range {start:Box::new(left.clone()),end:Box::new(right.clone())}),left.pos,left.length+2+right.length);
        }
        if let Node::Range(r) = &left.node {
            left
        }
        else{
            let er =self.raise_error("expected token 'range' after 'in' keyword");
            AstNode::new(Node::ParserError(er.clone()),er.pos,0)
        }
    }

    fn parse_while(&mut self) -> AstNode<Node> {
        let while_kw = match self.eat(&TokenType::Keyword(KeyWords::WHILE),"") {
            Ok(t)=>t,
            Err(e) => {return e;}
        };
        let loop_condition = self.parse_primary();
        // if self.expected(&TokenType::LeftParen){
        //     self.eat(&TokenType::LeftParen,"");
        //     loop_condition = self.parse_primary();
        //     match self.eat(&TokenType::RightParen,"unclosed parenthesis") {
        //         Ok(t)=>t,
        //         Err(e) => {return e;}
        //     };
            
        // }
        // else{
        //     loop_condition = self.parse_primary();
        // }
        // match self.eat(&TokenType::LeftBrace) {
        //     Ok(t)=>t,
        //     Err(e) => {return e;}
        // };
        let loop_body = self.parse_block();//self.parse_body(TokenType::RightBrace);
        // match self.eat(&TokenType::RightBrace) {
        //     Ok(t)=>t,
        //     Err(e) => {return e;}
        // };
        AstNode::new(
            Node::While(
                WhileLoop {
                    condition: Box::new(loop_condition),
                    body: loop_body 
                }
            ),
            while_kw.pos,
            0
        )
    }

    fn parse_paren(&mut self) -> AstNode<Node>{
        self.eat(&TokenType::LeftParen,"");
        let mut items:Vec<AstNode<Node>> = vec![];
        while !self.expected(&TokenType::RightParen) {
            items.push(self.parse_primary());
            if self.expected(&TokenType::Comma) {
                self.eat(&TokenType::Comma,"");
            }
        }
        let close = match self.eat(&TokenType::RightParen,"unclosed parenthesis"){
            Ok(t)=>t,
            Err(e) => {return e;}
        };
        if items.len() == 1 {
            return items[0].clone();
        }
        else{
            return AstNode::new(Node::Tuple(TupleBody { members: items.clone() }),items.first().unwrap().pos,close.length);
        }
    }

    fn parse_iden_init(&mut self,is_const : bool) -> AstNode<Node>{
        let declaration_token: AstNode<TokenType>;
        //let start_pos : (u32,u32);
        if is_const {
            declaration_token = match self.eat(&TokenType::Keyword(KeyWords::CONST),"") {
                Ok(t)=>t,
                Err(e) => {return e;}
            };
        }
        else {
            declaration_token = match self.eat(&TokenType::Keyword(KeyWords::LET),"") {
                Ok(t)=>t,
                Err(e) => {return e;}
            };
        }

        
        let var_name_token = match self.eat(&TokenType::Identifier(String::new()),"expected an identifier") {
            Ok(t)=>t,
            Err(e) => {return e;}
        };
        let var_name = if let TokenType::Identifier(id) = var_name_token.node {
            id
        }
        else{
            String::new()
        };
        

        match self.eat(&TokenType::Colon,"expected ':' for type declaration"){
            Ok(t) => t,
            Err(e) => {return e;}
        };
        let data_type = match self.parse_data_type() {
            Ok(t)=>t,
            Err(e) => {return e;}
        };

        let mut v = Var{
            constant:Some(AstNode::new(is_const, declaration_token.pos, declaration_token.length)),
            name:AstNode::new(
                var_name,
                var_name_token.pos,
                var_name_token.length
            ),
            var_type:AstNode::new(
                data_type.node.clone(),
                data_type.pos,
                data_type.length
            ),
        };
        if self.expected(&TokenType::AssignmentOperator){
            let operator = match self.eat(&TokenType::AssignmentOperator,"") {
                Ok(t)=>t,
                Err(e) => {return e;}
            };

            let out = Node::Assignment(BinExp 
                {
                    left: Box::from(
                            AstNode::new(
                                Node::DeclareVar(v.clone()), 
                                match &v.constant {
                                    Some(c)=> c.pos, 
                                    None => v.name.pos,
                                },
                                match &v.constant {
                                    Some(c)=> c.length, 
                                    None => v.name.length,
                                },
                            
                            )
                            
                        ),
                    right: Box::from(self.parse_primary()),
                    operator:operator,
                }
            );
            return AstNode::new(out,match &v.constant {
                Some(c)=> c.pos, 
                None => v.name.pos,
            },
            match &v.constant {
                Some(c)=> c.length, 
                None => v.name.length,
            })
            
        }
        return AstNode::new(
            Node::DeclareVar(v.clone()),
            match &v.constant {
                Some(c)=> c.pos, 
                None => v.name.pos,
            },
            match &v.constant {
                Some(c)=> c.length, 
                None => v.name.length,
            }
        );
    }

    fn parse_iden(&mut self) -> AstNode<Node>{
        let iden_token = match self.eat(&TokenType::Identifier(String::new()),"") {
            Ok(t) => t,
            Err(e) => {return e;}
        };
        
        let iden = if let TokenType::Identifier(id) = iden_token.node {
            id
        }
        else{
            String::new()
        };
        match &self.look_ahead.token{
            TokenType::LeftParen => {
                let mut func_call = FuncCall {function_name:AstNode::new(iden,iden_token.pos,iden_token.length), arguments:vec![]};
                let open_p = match self.eat(&TokenType::LeftParen,"") {
                    Ok(t)=>t,
                    Err(e) => {return e;}
                };
                
                while !self.expected(&TokenType::RightParen) {
                    if self.expected(&TokenType::EOF) {
                        //let er = self.raise_error("unclosed parenthesis");
                        self.err_pipe.raise_error(crate::ErrorType::SyntaxError, "unclosed parenthesis", open_p.pos, 1);
                        return AstNode::new(Node::ParserError(
                            ParserError::new(
                                ErrorType::SyntaxError,
                                "unclosed parenthesis",
                                open_p.pos
                            )
                        ),open_p.pos,1);
                    }
                    func_call.arguments.push(self.parse_primary());
                    if self.expected(&TokenType::Comma) {
                        self.eat(&TokenType::Comma,"");
                    }
                }
                match self.eat(&TokenType::RightParen,"") {
                    Ok(t)=>t,
                    Err(e) => {return e;}
                };
                AstNode::new(Node::FunctionCall(func_call),iden_token.pos,iden_token.length+3)
                // match self.var_map.get(&func_call.function_name){
                //     Some(_) => Node::FunctionCall(func_call),
                //     None => Node::ParserError(
                //         self.raise_error(ErrorType::SemanticError, format!("use of undeclared function {}",&func_call.function_name).as_str())
                //     ),
                // }
            },
            TokenType::AssignmentOperator => {
                // let var_info = match self.var_map.get(&iden){
                //     Some(info) => {
                //         info
                //     },
                //     None => {
                //         return  Node::ParserError(self.raise_error(ErrorType::SemanticError, format!("reassignment of undeclared variable {}",&iden).as_str()))
                //     },
                // };
                let operator = match self.eat(&TokenType::AssignmentOperator,"") {
                    Ok(t)=>t,
                    Err(e) => {return e;}
                };

                let right = self.parse_primary();
                let out = Node::Assignment(BinExp {
                    left: Box::from(AstNode::new(Node::Variable(iden),iden_token.pos,iden_token.length)),
                    right: Box::from(right.clone()), 
                    operator:operator.clone(),
                });
                
                AstNode::new(out,iden_token.pos,iden_token.length + operator.length + right.length)
            },
            TokenType::Dot => {
                // let mut caller = Node::Variable(match self.var_map.get(&iden) {
                //     Some(var) => UseVar {var_ref:var},
                //     None => return Node::ParserError(self.raise_error(ErrorType::SemanticError, format!("use of undeclared variable '{}'",&iden).as_str()))
                // });
                let mut caller = AstNode::new(Node::Variable(iden),iden_token.pos,iden_token.length);
                while self.expected(&TokenType::Dot) {
                    self.eat(&TokenType::Dot,"");
                    let method_token = match self.eat(&TokenType::Identifier(String::new()),"expected field name or method") {
                        Ok(t)=>t,
                        Err(e) => {return e;}
                    };
                    let method_iden = if let TokenType::Identifier(id) = method_token.node {
                        id
                    }
                    else{
                        String::new()
                    };
                    let mut method_call = MethodCall { caller: None, method_name: AstNode::new(method_iden,method_token.pos,method_token.length), arguments: vec![] };
                    match self.eat(&TokenType::LeftParen,"expected arguments") {
                        Ok(t)=>t,
                        Err(e) => {return e;}
                    };
                    while !self.expected(&TokenType::RightParen) {
                        if self.expected(&TokenType::EOF){
                            self.raise_error("unclosed parenthesis");
                        }
                        method_call.arguments.push(self.parse_primary());
                        if self.expected(&TokenType::Comma) {
                            self.eat(&TokenType::Comma,"");
                        }
                    }
                    match self.eat(&TokenType::RightParen,"") {
                        Ok(t)=>t,
                        Err(e) => {return e;}
                    };
                    method_call.caller = Some(Box::from(caller));
                    caller = AstNode::new(Node::MethodCall(method_call),method_token.pos,method_token.length);
                    
                }
                caller
            },
            _ => {
                AstNode::new(Node::Variable(iden),iden_token.pos,iden_token.length)
            },
        }
    }
    fn parse_primary(&mut self) -> AstNode<Node> {
        let mut left = self.parse_compose();
        match &self.look_ahead.token{
            TokenType::Or =>{
                while self.expected(&TokenType::Or){
                    let operator = match self.eat(&TokenType::Or,"") {
                        Ok(t)=>t,
                        Err(e) => {return e;}
                    };
                    let right = self.parse_compose();
                    left = AstNode::new(
                        Node::BinaryExpression(
                            BinExp {
                                left:Box::from(left.clone()),
                                right:Box::from(right.clone()),
                                operator:operator.clone()
                            }
                        ),
                        left.pos,
                        left.length + operator.length + right.length
                    );
                }
            },
            _ => {

            }
        };
        return left;
    }

    fn parse_compose(&mut self) -> AstNode<Node> {
        let mut left = self.parse_boolean_expression();
        match &self.look_ahead.token{
            TokenType::And =>{
                while self.expected(&TokenType::And){
                    let operator = match self.eat(&TokenType::And,"") {
                        Ok(t)=>t,
                        Err(e) => {return e;}
                    };
                    let right = self.parse_boolean_expression();
                    left = AstNode::new(
                        Node::BinaryExpression(
                            BinExp {
                                left:Box::from(left.clone()),
                                right:Box::from(right.clone()),
                                operator:operator.clone()
                            }
                        ),
                        left.pos,
                        left.length + operator.length + right.length
                    );
                }
            },
            _ => {

            }
        };
        return left;
    }

    fn parse_boolean_expression(&mut self) -> AstNode<Node> {
        let mut left = self.parse_arithmatic();
        loop{
            match &self.look_ahead.token{
                TokenType::More | TokenType::MoreEqual | TokenType::Equal | TokenType::Less | TokenType::LessEqual =>{
                        let operator = match self.eat(&self.look_ahead.token.clone(),"") {
                            Ok(t)=>t,
                            Err(e) => {return e;}
                        };
                        let right = self.parse_arithmatic();
                        left = AstNode::new(
                            Node::BinaryExpression(
                                BinExp {
                                    left:Box::from(left.clone()),
                                    right:Box::from(right.clone()),
                                    operator:operator.clone()
                                }
                            ),
                            left.pos,
                            left.length + operator.length + right.length
                        );
                },
                _ => {
                    break;
                }
            };
        }
        
        return left;
    }

    fn parse_arithmatic(&mut self) -> AstNode<Node> {
        let mut left = self.parse_term();
        match &self.look_ahead.token{
            TokenType::Range =>{
                if self.expected(&TokenType::Range){
                    let operator = match self.eat(&TokenType::Range,"") {
                        Ok(t)=>t,
                        Err(e) => {return e;}
                    };
                    let right = self.parse_term();
                    left = AstNode::new(
                        Node::Range(
                            Range {
                                start:Box::from(left.clone()),
                                end:Box::from(right.clone()),
                            }
                        ),
                        left.pos,
                        left.length + operator.length + right.length
                    );
                }
            },
            TokenType::AdditionOperator | TokenType::SubtractionOperator =>{
                while self.expected(&TokenType::AdditionOperator) || self.expected(&TokenType::SubtractionOperator){
                    let operator = match self.eat(&self.look_ahead.token.clone(),"") {
                        Ok(t)=>t,
                        Err(e) => {return e;}
                    };
                    let right = self.parse_term();
                    left = AstNode::new(
                        Node::BinaryExpression(
                            BinExp {
                                left:Box::from(left.clone()),
                                right:Box::from(right.clone()),
                                operator:operator.clone()
                            }
                        ),
                        left.pos,
                        left.length + operator.length + right.length
                    );
                }
            },
            _ => {

            }

        }
        
        return left;
    }

    fn parse_term(&mut self) -> AstNode<Node> {
        let mut left = self.parse_factor();
        match self.look_ahead.token{
            TokenType::MultiplicationOperator | TokenType::DivisionOperator | TokenType::ModuloOperator =>{
                while self.expected(&TokenType::MultiplicationOperator) || self.expected(&TokenType::DivisionOperator) || self.expected(&TokenType::ModuloOperator){
                    let operator = match self.eat(&self.look_ahead.token.clone(),"") {
                        Ok(t)=>t,
                        Err(e) => {return e;}
                    };
                    let right = self.parse_factor();
                    left = AstNode::new(
                        Node::BinaryExpression(
                            BinExp {
                                left:Box::from(left.clone()),
                                right:Box::from(right.clone()),
                                operator:operator.clone()
                            }
                        ),
                        left.pos,
                        left.length + operator.length + right.length
                    );
                }
            },
            _ => {

            }

        }
        
        return left;
        
    }

    fn parse_factor(&mut self) -> AstNode<Node> {
        let node = match &self.look_ahead.token.clone() {
            TokenType::Identifier(id) => {
                // let var_info = match self.var_map.get(&id){
                //     Some(info) => {
                //         match info {
                //             Node::DeclareVar(var) => {
                //                 var
                //             }
                //             _ => {

                //             }
                //         }
                //         println!("found {:#?}",info);
                //         info
                //     },
                //     None => {
                //         return (Node::ParserError(self.raise_error(ErrorType::SemanticError, format!("use of undeclared variable '{}'",&id).as_str())),DataType::Void)
                //     },
                // };
                self.parse_iden()
                //self.eat(&TokenType::Identifier(String::new()),"");
            },
            TokenType::IntLiteral(i) => {
                let int = match self.eat(&TokenType::IntLiteral(0),"") {
                    Ok(t)=>t,
                    Err(e) => {return e;}
                };
                AstNode::new(Node::Literal(LiteralValue::Int(*i)),int.pos,int.length)
            },
            TokenType::FloatLiteral(f) => {
                let flt = match self.eat(&TokenType::FloatLiteral(0.0),"") {
                    Ok(t)=>t,
                    Err(e) => {return e;}
                };
                
                AstNode::new(Node::Literal(LiteralValue::Float(*f)),flt.pos,flt.length)
            },
            TokenType::StringLiteral(s) => {
                let str = match self.eat(&TokenType::StringLiteral(String::new()),"") {
                    Ok(t)=>t,
                    Err(e) => {return e;}
                };
                AstNode::new(Node::Literal(LiteralValue::Str(s.clone())),str.pos,str.length)
            },
            TokenType::BooleanLiteral(b) => {
                let boolean = match self.eat(&TokenType::BooleanLiteral(true),"") {
                    Ok(t)=>t,
                    Err(e) => {return e;}
                };
                AstNode::new(Node::Literal(LiteralValue::Bool(*b)),boolean.pos,boolean.length)
            },
            TokenType::LeftParen =>{
                self.eat(&TokenType::LeftParen,"");
                let out = self.parse_primary();
                match self.eat(&TokenType::RightParen,"unclosed parenthesis") {
                    Ok(t)=>t,
                    Err(e) => {return e;}
                };
                out
            },
            TokenType::Not =>{
                self.eat(&TokenType::Not,"");
                let out = self.parse_primary();
                
                AstNode::new(Node::BooleanNot(NotExp { exp: Box::new(out.clone()) }), out.pos, out.length)
            },
            TokenType::SubtractionOperator => {
                self.eat(&TokenType::SubtractionOperator,"");
                match self.look_ahead.token{
                    TokenType::IntLiteral(i) => {
                        let int = match self.eat(&TokenType::IntLiteral(0),"") {
                            Ok(t)=>t,
                            Err(e) => {return e;}
                        };
                        AstNode::new(Node::Literal(LiteralValue::Int(i*-1)),int.pos,int.length)
                    },
                    TokenType::FloatLiteral(f) => {
                        let flt = match self.eat(&TokenType::FloatLiteral(0.0),"") {
                            Ok(t)=>t,
                            Err(e) => {return e;}
                        };
                        AstNode::new(Node::Literal(LiteralValue::Float(f*-1.0)),flt.pos,flt.length)
                    },
                    
                    _ => {
                        let er = self.raise_error("expected expression after '-' token");
                        
                        AstNode::new(
                            Node::ParserError(
                                er.clone()
                            ),
                            er.pos,
                            0
                        )
                    }
                }
                
            },
            
            _ => {
                let er = self.raise_error("expected expression");
    
                AstNode::new(
                    Node::ParserError(
                        er.clone()
                    ),
                    er.pos,
                    0
                )
            }
        };
        node
        
    }

    pub fn parse(&mut self) -> Body {
        let program = self.parse_body(TokenType::EOF);
        program
    }

}

