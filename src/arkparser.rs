pub mod tokenizer;

use crate::tokenizer::{Tokenizer,DataType, KeyWords, Token};
use std::{collections::HashMap, mem};


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
    Return(Option<Box<Node>>),
    Error(String),
}

#[derive(Debug,Clone)]
pub enum LiteralValue {
    Int(i32),
    Float(f64),
    Str(String)
}

#[derive(Debug,Clone)]
pub struct Body {
    pub instructions: Vec<Node>,
}

#[derive(Debug,Clone)]
pub struct BinExp {
    pub left: Box<Node>,
    pub right: Box<Node>,
    pub operator: Token,
}

#[derive(Debug,Clone)]
pub struct Var {
    pub constant:bool,
    pub name: String,
    pub var_type: DataType,
}

#[derive(Debug,Clone)]
pub struct FuncDef {
    pub function_name: String,
    pub body:Body,
    pub return_type: DataType,
    pub parameter: Vec<Var>,
}

#[derive(Debug,Clone)]
pub struct FuncCall {
    pub function_name: String,
    pub arguments: Vec<Node>,
}

#[derive(Debug,Clone)]
pub struct MethodCall {
    pub caller:Option<Box<Node>>,
    pub method_name: String,
    pub arguments: Vec<Node>,
}

#[derive(Debug,Clone)]
pub struct Import {
    pub import_name: String,
    pub alias: Option<String>,
}

pub struct ArkParser<'a> {
    tokenizer: Tokenizer<'a>,
    look_ahead: Token,
    data_type_map: HashMap<String,DataType>,
    iden_map: HashMap<String,Node>
}

impl<'a> ArkParser<'a> {
    pub fn new(source_code: &'a str) -> ArkParser<'a> {
        let mut parser_tokenizer = Tokenizer::new(source_code);
        
        return ArkParser {
            look_ahead: parser_tokenizer.get_next_token(),
            tokenizer: parser_tokenizer,
            data_type_map:HashMap::from([
                ("u8".to_owned(),DataType::U8),
                ("u16".to_owned(),DataType::U16),
            ]),
            iden_map:HashMap::new(),
            
        };
    }

    fn expected(&mut self, expected: Token) -> bool {
        return self.look_ahead == expected;
    }

    fn eat(&mut self, expected: Token) -> Token {
        match expected{
            Token::Keyword(keyword) =>{
                let temp = self.look_ahead.clone();
                if mem::discriminant(&self.look_ahead) == mem::discriminant(&expected) {
                    match &self.look_ahead{
                        Token::Keyword(look_ahead_key) => {
                            if mem::discriminant(look_ahead_key) == mem::discriminant(&keyword) {
                                self.look_ahead = self.tokenizer.get_next_token();
                                return temp;
                            }
                            panic!(
                                "Unexpected token of type {:?} (expected: {:?})",
                                temp, expected
                            );
                        },
                        _ => {
                            panic!(
                                "Unexpected token of type {:?} (expected: {:?})",
                                temp, expected
                            );
                        },
                    }
                    
                }
                panic!(
                    "Unexpected token of type {:?} (expected: {:?})",
                    temp, expected
                );
            },
            _ => {
                let temp = self.look_ahead.clone();
                if mem::discriminant(&temp) == mem::discriminant(&expected) {
                    self.look_ahead = self.tokenizer.get_next_token();
                    return temp;
                }
                panic!(
                    "Unexpected token of type {:?} (expected: {:?})",
                    temp, expected
                );
            }
        }
        
        
    }
    fn parse_body(&mut self,stop_token:Token) -> Body {
        let mut scope_body = Body {
            instructions: vec![],
        };
        while self.look_ahead != stop_token {
            let node:Node = match &self.look_ahead {
                Token::Keyword(keyword) => {
                    match keyword {
                        KeyWords::FUNC => {
                            self.parse_function()
                        },
                        KeyWords::IMPORT => {
                            self.parse_import()
                        },
                        KeyWords::AS => {
                            panic!("unexpected keyword \"as\"");
                        },
                        KeyWords::CONST => {
                            self.parse_iden_init()
                        },
                        KeyWords::RETURN => {
                            self.parse_return()
                        },
                    }
                },
                Token::Identifier(_) => {
                    self.parse_iden()
                },
                Token::DataType(_) => {
                    self.parse_iden_init()
                }
                Token::MultiplicationOperator => todo!(),
                _ => {Node::Error("Unrecognize".to_string())},
            };
            scope_body.instructions.push(node);
        }
        return scope_body;
    }
    fn parse_import(&mut self) -> Node {
        let _import_keyword = self.eat(Token::Keyword(KeyWords::IMPORT));
        let import_name = self.eat(Token::StringLiteral(String::new()));
        if self.expected(Token::Keyword(KeyWords::AS)) {
            let _import_as = self.eat(Token::Keyword(KeyWords::AS));
            let import_alias = self.eat(Token::Identifier(String::new()));
            let _import_semi = self.eat(Token::SemiColon);
            return Node::Import(Import {
                import_name: if let Token::StringLiteral(s) = import_name {
                    s
                } else {
                    String::new()
                },
                alias: if let Token::Identifier(id) = import_alias {
                    Some(id)
                } else {
                    None
                },
            });
        }
        let _import_semi = self.eat(Token::SemiColon);
        return Node::Import(Import {
            import_name: if let Token::StringLiteral(s) = import_name {
                s
            } else {
                String::new()
            },
            alias: None,
        });
    }
    fn parse_return(&mut self) -> Node {
        self.eat(Token::Keyword(KeyWords::RETURN));
        if self.expected(Token::SemiColon) {
            self.eat(Token::SemiColon);
            return Node::Return(None);
        }
        
        let val = Node::Return(Some(Box::from(self.parse_primary())));
        self.eat(Token::SemiColon);
        val
    }
    fn parse_function(&mut self) -> Node {
        let _func_keyword = self.eat(Token::Keyword(KeyWords::FUNC));
        let func_name = self.eat(Token::Identifier(String::new()));
        let _ = self.eat(Token::LeftParen);
        let mut parameters: Vec<Var> = vec![];
        while !self.expected(Token::RightParen){
            let para_type_t = self.eat(Token::DataType(DataType::Void));
            let para_type = if let Token::DataType(data_type) = para_type_t {
                data_type
            }
            else{
                DataType::Void
            };
            let para_name_t = self.eat(Token::Identifier(String::new()));
            let para_name = if let Token::Identifier(id) = para_name_t {
                id
            }
            else{
                String::new()
            };
            parameters.push(Var {constant:false,name:para_name,var_type:para_type});
            if self.expected(Token::Comma) {
                self.eat(Token::Comma);
            }
        }
        let _ = self.eat(Token::RightParen);
        let mut return_type = DataType::Void;
        if self.expected(Token::Colon) {
            let _ = self.eat(Token::Colon);
            let return_token = self.eat(Token::DataType(DataType::String));
            return_type = if let Token::DataType(data_type) = return_token {
                data_type
            }
            else{
                DataType::Void
            };
        }
        let _ = self.eat(Token::LeftBrace);
        
        let function_body = if !self.expected(Token::RightBrace) {
            self.parse_body(Token::RightBrace)
        }
        else{
            Body {instructions:vec![]}
        };
        let _ = self.eat(Token::RightBrace);
        let function_name = if let Token::Identifier(id) = func_name {
            id
        } else {
            String::new()
        };
        self.iden_map.insert(function_name.clone(), Node::Function(
            FuncDef {
                function_name: function_name.clone(),
                body:function_body.clone(),
                parameter:parameters.clone(),
                return_type:return_type.clone()
            }
        ));
        return Node::Function(
            FuncDef {
                function_name: function_name.clone(),
                body:function_body.clone(),
                parameter:parameters,
                return_type:return_type.clone()
            }
        );

    }
    fn parse_iden_init(&mut self) -> Node{
        let mut constant = false;
        if self.expected(Token::Keyword(KeyWords::CONST)){
            self.eat(Token::Keyword(KeyWords::CONST));
            constant = true;
        }
        let data_type_token = self.eat(Token::DataType(DataType::Void));
        let data_type = if let Token::DataType(data_type) = data_type_token {
            data_type
        }
        else{
            DataType::Void
        };
        let var_name_token = self.eat(Token::Identifier(String::new()));
        let var_name = if let Token::Identifier(id) = var_name_token {
            id
        }
        else{
            String::new()
        };
        if self.expected(Token::AssignmentOperator){
            let operator = self.eat(Token::AssignmentOperator);
            let out = Node::Assignment(BinExp 
                {
                    left: Box::from(
                            Node::DeclareVar(
                                Var{
                                    constant,
                                    name:var_name,
                                    var_type:data_type
                                }
                            )
                        ),
                    right: Box::from(self.parse_primary()),
                    operator
                }
            );
            self.eat(Token::SemiColon);
            return out; 
        }
        self.eat(Token::SemiColon);
        let v = Var{
            constant,
            name:var_name,
            var_type:data_type
        };
        self.iden_map.insert(v.name.clone(), Node::DeclareVar(
            v.clone()
        ));
        return Node::DeclareVar(
            v.clone()
        );
    }

    fn parse_iden(&mut self) -> Node{
        let iden_token = self.eat(Token::Identifier(String::new()));
        let iden = if let Token::Identifier(id) = iden_token {
            id
        }
        else{
            String::new()
        };
        match &self.look_ahead{
            Token::LeftParen => {
                let mut func_call = FuncCall {function_name:iden, arguments:vec![]};
                self.eat(Token::LeftParen);
                while !self.expected(Token::RightParen){
                    func_call.arguments.push(self.parse_primary());
                    if self.expected(Token::Comma) {
                        self.eat(Token::Comma);
                    }
                }
                self.eat(Token::RightParen);
                self.eat(Token::SemiColon);
                Node::FunctionCall(func_call)
            },
            Token::AssignmentOperator => {
                let operator = self.eat(Token::AssignmentOperator);
                let out = Node::Assignment(BinExp {
                    left: Box::from(Node::Variable(iden)),
                    right: Box::from(self.parse_primary()), 
                    operator
                });
                self.eat(Token::SemiColon);
                out
            },
            Token::Dot => {
                let mut caller = self.iden_map.get(&iden).unwrap().clone();
                while self.expected(Token::Dot) {
                    self.eat(Token::Dot);
                    let method_token = self.eat(Token::Identifier(String::new()));
                    let method_iden = if let Token::Identifier(id) = method_token {
                        id
                    }
                    else{
                        String::new()
                    };
                    let mut method_call = MethodCall { caller: None, method_name: method_iden, arguments: vec![] };
                    self.eat(Token::LeftParen);
                    while !self.expected(Token::RightParen) {
                        method_call.arguments.push(self.parse_primary());
                        if self.expected(Token::Comma) {
                            self.eat(Token::Comma);
                        }
                    }
                    self.eat(Token::RightParen);
                    method_call.caller = Some(Box::from(caller));
                    caller = Node::MethodCall(method_call);
                    
                }
                self.eat(Token::SemiColon);
                caller
            },
            _ => Node::Variable(iden),
        }
    }

    fn parse_primary(&mut self) -> Node {
        let mut left:Node = self.parse_term();
        while self.expected(Token::AdditionOperator) || self.expected(Token::SubtractionOperator){
            let operator = self.eat(self.look_ahead.clone());
            let right:Node = self.parse_term();
            left = Node::BinaryExpression(BinExp { left:Box::from(left), right:Box::from(right), operator });
        }
        return left;
    }

    fn parse_term(&mut self) -> Node {
        let mut left:Node = self.parse_factor();
        while self.expected(Token::MultiplicationOperator) || self.expected(Token::DivisionOperator) || self.expected(Token::ModuloOperator){
            let operator = self.eat(self.look_ahead.clone());
            let right:Node = self.parse_factor();
            left = Node::BinaryExpression(BinExp { left:Box::from(left), right:Box::from(right), operator });
        }
        return left;
        
    }

    fn parse_factor(&mut self) -> Node {
        match &self.look_ahead.clone() {
            Token::Identifier(id) => {
                self.eat(Token::Identifier(String::new()));
                Node::Variable(id.to_owned())
            },
            Token::IntLiteral(i) => {
                self.eat(Token::IntLiteral(0));
                Node::Literal(LiteralValue::Int(*i))
            },
            Token::FloatLiteral(f) => {
                self.eat(Token::FloatLiteral(0.0));
                Node::Literal(LiteralValue::Float(*f))
            },
            Token::StringLiteral(s) => {
                self.eat(Token::StringLiteral(String::new()));
                Node::Literal(LiteralValue::Str(s.clone()))
            },
            Token::LeftParen =>{
                self.eat(Token::LeftParen);
                let out = self.parse_primary();
                self.eat(Token::RightParen);
                out
            },
            Token::SubtractionOperator => {
                self.eat(Token::SubtractionOperator);
                match &self.look_ahead.clone(){
                    Token::IntLiteral(i) => {
                        self.eat(Token::IntLiteral(0));
                        Node::Literal(LiteralValue::Int(*i*-1))
                    },
                    Token::FloatLiteral(f) => {
                        self.eat(Token::FloatLiteral(0.0));
                        Node::Literal(LiteralValue::Float(*f*-1.0))
                    },
                    
                    _ => {
                        Node::Error("Expected value after return keyword".to_string())
                    },
                }
                
            },
            
            _ => {
                Node::Error("Expected value after return keyword".to_string())
            },
        }
        
    }

    pub fn parse(&mut self) -> Vec<Node> {
        self.parse_body(Token::EOF).instructions
    }
}
