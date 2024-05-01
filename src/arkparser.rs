use self::tokenizer::{DataType, KeyWords, Token};
use std::{collections::HashMap, mem, ops::Deref, str::FromStr};
mod tokenizer;

#[derive(Debug,Clone)]
enum Node {
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

// impl Node {
//     fn clone(&self) -> Node {
//         match self{
//             Node::Body(bd) => {Node::Body(Body { instructions: bd.instructions })},
//             Node::Variable(v) => {Node::Variable(v.to_owned())},
//             Node::DeclareVar(v) => {Node::DeclareVar(Var { constant: v.constant.to_owned(), name: v.name.to_owned(), var_type: v.var_type.clone() })},
//             Node::Assignment(a) => {Node::Assignment(a.clone())},
//             Node::Literal(l) => {
//                 let v = match l{
//                     LiteralValue::Int(I) => {LiteralValue::Int(*I)},
//                     LiteralValue::Float(F) => {LiteralValue::Float(*F)},
//                     LiteralValue::Str(S) => {LiteralValue::Str(*S)},
//                 };
//                 Node::Literal(v)
//             },
//             Node::BinaryExpression(b) => {Node::BinaryExpression(b.clone())},
//             Node::Function(f) => {
//                 Node::Function(
//                     FuncDef {
//                         function_name: f.function_name.clone(),
//                         body: Body { instructions: f.body.instructions.to_vec() },
//                         return_type: (),
//                         parameter: () 
//                     }
//                 )
//             },
//             Node::FunctionCall(f) => todo!(),
//             Node::MethodCall(_) => todo!(),
//             Node::Import(_) => todo!(),
//             Node::Return(_) => todo!(),
//             Node::Error(_) => todo!(),
//         }
//     }
// }

#[derive(Debug,Clone)]
enum LiteralValue {
    Int(i32),
    Float(f64),
    Str(String)
}

#[derive(Debug,Clone)]
struct Body {
    instructions: Vec<Node>,
}

#[derive(Debug,Clone)]
struct BinExp {
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
    operator: tokenizer::Token,
}

// impl BinExp {
//     pub fn clone(&self) -> BinExp {
//         BinExp {
//             left: Some(Box::new((*self.left.unwrap()).clone())),
//             right: Some(Box::new((*self.left.unwrap()).clone())),
//             operator: self.operator.clone()
//         }
//     }
// }

#[derive(Debug,Clone)]
struct Var {
    constant:bool,
    name: String,
    var_type: DataType,
}

#[derive(Debug,Clone)]
struct FuncDef {
    function_name: String,
    body:Body,
    return_type: tokenizer::DataType,
    parameter: Vec<Var>,
}

#[derive(Debug,Clone)]
struct FuncCall {
    function_name: String,
    arguments: Vec<Node>,
}

#[derive(Debug,Clone)]
struct MethodCall {
    caller:Option<Box<Node>>,
    method_name: String,
    arguments: Vec<Node>,
}

#[derive(Debug,Clone)]
struct Import {
    import_name: String,
    alias: Option<String>,
}

pub struct ArkParser<'a> {
    tokenizer: tokenizer::Tokenizer<'a>,
    ast: Option<Box<Node>>,
    look_ahead: tokenizer::Token,
    data_type_map: HashMap<String,DataType>,
    iden_map: HashMap<String,Node>
}

impl<'a> ArkParser<'a> {
    pub fn new(source_code: &'a str) -> ArkParser<'a> {
        let mut parser_tokenizer = tokenizer::Tokenizer::new(source_code);
        
        return ArkParser {
            look_ahead: parser_tokenizer.get_next_token(),
            tokenizer: parser_tokenizer,
            ast: None,
            data_type_map:HashMap::from([
                ("u8".to_owned(),DataType::U8),
                ("u16".to_owned(),DataType::U16),
            ]),
            iden_map:HashMap::new(),
            
        };
    }

    fn expected(&mut self, expected: tokenizer::Token) -> bool {
        return self.look_ahead == expected;
    }

    fn eat(&mut self, expected: tokenizer::Token) -> Token {
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
    fn parse_body(&mut self,stop_token:tokenizer::Token) -> Body {
        let mut scope_body = Body {
            instructions: vec![],
        };
        while self.look_ahead != stop_token {
            let mut node:Node = match &self.look_ahead {
                Token::Keyword(keyword) => {
                    match keyword {
                        KeyWords::FUNC => {
                            println!("Parse func");
                            self.parse_function()
                        },
                        KeyWords::IMPORT => {
                            println!("Parse import");
                            self.parse_import()
                        },
                        KeyWords::AS => {
                            panic!("unexpected keyword \"as\"");
                        },
                        KeyWords::CONST => {
                            self.parse_iden_init()
                        },
                        KeyWords::RETURN => {
                            self.eat(Token::Keyword(KeyWords::RETURN));
                            Node::Return(Some(Box::from(self.parse_primary())))
                        },
                    }
                },
                Token::Identifier(id) => {
                    self.parse_iden()
                },
                Token::DataType(dt) => {
                    println!("Parse var init");
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
        let _import_keyword = self.eat(tokenizer::Token::Keyword(KeyWords::IMPORT));
        let import_name = self.eat(tokenizer::Token::StringLiteral(String::new()));
        if self.expected(tokenizer::Token::Keyword(KeyWords::AS)) {
            let _import_as = self.eat(tokenizer::Token::Keyword(KeyWords::AS));
            let import_alias = self.eat(tokenizer::Token::Identifier(String::new()));
            let _import_semi = self.eat(tokenizer::Token::SemiColon);
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
        let _import_semi = self.eat(tokenizer::Token::SemiColon);
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
            return Node::Return(None);
        }
        return Node::Return(Some(Box::from(self.parse_primary())));
    }
    
    fn parse_function(&mut self) -> Node {
        let _func_keyword = self.eat(tokenizer::Token::Keyword(KeyWords::FUNC));
        let func_name = self.eat(tokenizer::Token::Identifier(String::new()));
        let _ = self.eat(tokenizer::Token::LeftParen);
        let _ = self.eat(tokenizer::Token::RightParen);
        let mut return_type = DataType::Void;
        if self.expected(tokenizer::Token::Colon) {
            let _ = self.eat(tokenizer::Token::Colon);
            let return_token = self.eat(tokenizer::Token::DataType(tokenizer::DataType::String));
            return_type = if let Token::DataType(data_type) = return_token {
                data_type
            }
            else{
                DataType::Void
            };
        }
        let _ = self.eat(tokenizer::Token::LeftBrace);
        
        let function_body = if !self.expected(tokenizer::Token::RightBrace) {
            self.parse_body(Token::RightBrace)
        }
        else{
            Body {instructions:vec![]}
        };
        let _ = self.eat(tokenizer::Token::RightBrace);
        let function_name = if let Token::Identifier(id) = func_name {
            id
        } else {
            String::new()
        };
        self.iden_map.insert(function_name.clone(), Node::Function(
            FuncDef {
                function_name: function_name.clone(),
                body:function_body.clone(),
                parameter:vec![],
                return_type:return_type.clone()
            }
        ));
        return Node::Function(
            FuncDef {
                function_name: function_name.clone(),
                body:function_body.clone(),
                parameter:vec![],
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
        println!("Inside init");
        if self.expected(Token::AssignmentOperator){
            let operator = self.eat(Token::AssignmentOperator);
            let out = Node::Assignment(BinExp 
                {
                    left: Some(
                        Box::from(
                            Node::DeclareVar(
                                Var{
                                    constant,
                                    name:var_name,
                                    var_type:data_type
                                }
                            )
                        )
                    ),
                    right: Some(Box::from(self.parse_primary())),
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
                    left: Some(Box::from(Node::Variable(iden))),
                    right: Some(Box::from(self.parse_primary())), 
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
        let mut left:Node = match &self.look_ahead.clone() {
            Token::Identifier(_) => {
                self.parse_iden()
            },
            Token::IntLiteral(i) => {
                self.eat(Token::IntLiteral(0));
                println!("Found Int Literal");
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
            _ => {
                Node::Error("Expected value after return keyword".to_string())
            },
        };
        while self.expected(Token::AdditionOperator) || self.expected(Token::SubtractionOperator){
            let operator = self.eat(self.look_ahead.clone());
            let right:Node = match &self.look_ahead.clone() {
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
                _ => {
                    Node::Error("Expected value after return keyword".to_string())
                },
            };
            left = Node::BinaryExpression(BinExp { left:Some(Box::from(left)), right:Some(Box::from(right)), operator });
        }
        return left;
    }

    fn parse_factor(&mut self) -> Node {
        let mut left:Node = match &self.look_ahead.clone() {
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
            _ => {
                Node::Error("Expected value after return keyword".to_string())
            },
        };
        while self.expected(Token::MultiplicationOperator) || self.expected(Token::DivisionOperator){
            let operator = self.eat(self.look_ahead.clone());
            let right:Node = match &self.look_ahead.clone() {
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
                _ => {
                    Node::Error("Expected value after return keyword".to_string())
                },
            };
            left = Node::BinaryExpression(BinExp { left:Some(Box::from(left)), right:Some(Box::from(right)), operator });
        }
        return left;
        
    }

    pub fn parse(&mut self) {
        let program = self.parse_body(Token::EOF);


        // self.ast = Some(Box::new(self.parse_import()));
        println!("{:?}", program);

        // while !self.tokenizer.is_finished(){
        //     let token = self.tokenizer.get_next_token();
        //     match token {
        //         tokenizer::Token::Keyword(keyword) => {
        //             match keyword {
        //                 tokenizer::KeyWords::FUNC => {
        //                     println!("KeyWord: func");
        //                 },
        //                 tokenizer::KeyWords::IMPORT => {
        //                     println!("KeyWord: import");
        //                 },
        //                 tokenizer::KeyWords::AS => {
        //                     println!("KeyWord: as");
        //                 },
        //                 tokenizer::KeyWords::CONST => {
        //                     println!("KeyWord: const");
        //                 },
        //                 tokenizer::KeyWords::RETURN => {
        //                     println!("KeyWord: return");
        //                 },
        //             }
        //         },
        //         tokenizer::Token::Identifier(id) => {
        //             println!("Identifier: {}",id);
        //         },
        //         tokenizer::Token::IntLiteral(int) => {
        //             println!("Int: {}",int);
        //         },
        //         tokenizer::Token::FloatLiteral(float) => {
        //             println!("Float: {}",float);
        //         },
        //         tokenizer::Token::StringLiteral(s) => {
        //             println!("String: {}",s);
        //         },
        //         tokenizer::Token::AdditionOperator => {
        //             println!("Addition: +");
        //         },
        //         tokenizer::Token::SubtractionOperator => {
        //             println!("Subtraction: -");
        //         },
        //         tokenizer::Token::MultiplicationOperator => {
        //             println!("Multiplication: *");
        //         },
        //         tokenizer::Token::DivisionOperator => {
        //             println!("Division: *");
        //         },
        //         tokenizer::Token::ModuloOperator => {
        //             println!("Modulo: %");
        //         },
        //         tokenizer::Token::LeftParen => {
        //             println!("LeftParenthesis: (")
        //         },
        //         tokenizer::Token::RightParen => {
        //             println!("RightParenthesis: )")
        //         },
        //         tokenizer::Token::LeftBrace => {
        //             println!("LeftBrace: {{")
        //         },
        //         tokenizer::Token::RightBrace => {
        //             println!("RightBrace: }}")
        //         },
        //         tokenizer::Token::Comma => {
        //             println!("Comma: ,")
        //         },
        //         tokenizer::Token::Dot => {
        //             println!("Dot: .")
        //         },
        //         tokenizer::Token::SemiColon => {
        //             println!("SemiColon: ;")
        //         },
        //         tokenizer::Token::EOF => {
        //             println!("Reach End of File")
        //         },
        //         tokenizer::Token::AssignmentOperator => {
        //             println!("AssignmentOperator: =")
        //         },
        //         tokenizer::Token::Equal => {
        //             println!("Equal: ==")
        //         },
        //         tokenizer::Token::Less => {
        //             println!("Less: <")
        //         },
        //         tokenizer::Token::LessEqual => {
        //             println!("LessEqual: <=")
        //         },
        //         tokenizer::Token::More => {
        //             println!("More: >")
        //         },
        //         tokenizer::Token::MoreEqual => {
        //             println!("MoreEqual: >=")
        //         },
        //     }
        // }
    }

    fn parse_format(&mut self){
        let program = self.parse_body(Token::EOF);
    }
}
