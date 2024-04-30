use self::tokenizer::{DataType, KeyWords, Token};
use std::{collections::HashMap, mem};
mod tokenizer;

struct ASTNode {}

#[derive(Debug)]
enum Node {
    Body(body),
    Variable(var),
    BinaryExpression(BinExp),
    Function(func),
    Import(import),
}

#[derive(Debug)]
struct body {
    instructions: Vec<Box<Node>>,
}

#[derive(Debug)]
struct BinExp {
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
    operator: tokenizer::Token,
}

#[derive(Debug)]
struct var {
    constant:bool,
    name: String,
    var_type: DataType,
}
#[derive(Debug)]
struct func {
    function_name: String,
    body:body,
    return_type: tokenizer::DataType,
    parameter: Vec<var>,
}
#[derive(Debug)]
struct import {
    import_name: String,
    alias: Option<String>,
}

pub struct ArkParser<'a> {
    tokenizer: tokenizer::Tokenizer<'a>,
    ast: Option<Box<Node>>,
    look_ahead: tokenizer::Token,
    data_type_map: HashMap<String,DataType>
}
impl<'a> ArkParser<'a> {
    pub fn new(source_code: &'a str) -> ArkParser<'a> {
        let mut parser_tokenizer = tokenizer::Tokenizer::new(source_code);
        
        return ArkParser {
            look_ahead: parser_tokenizer.get_next_token(),
            tokenizer: parser_tokenizer,
            ast: None,
            data_type_map:HashMap::new<String,DataType>({
                ("u8",DataType::U8)
            }),
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
                    match self.look_ahead{
                        Token::Keyword(look_ahead_key) => {
                            if mem::discriminant(&look_ahead_key) == mem::discriminant(&keyword) {
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
    fn parse_body(&mut self) -> body {
        let mut scope_body = body {
            instructions: vec![],
        };
        while self.look_ahead != Token::EOF {
            let mut node:Node = match &self.look_ahead {
                Token::Keyword(keyword) => {
                    match keyword {
                        KeyWords::FUNC => {
                            return self.parse_function();
                        },
                        KeyWords::IMPORT => {
                            return self.parse_import();
                        },
                        KeyWords::AS => {
                            panic!("unexpected keyword \"as\"");
                        },
                        KeyWords::CONST => {
                            return self.parse_var_init();
                        },
                        KeyWords::RETURN => {
                            self.eat(Token::Keyword(KeyWords::RETURN));
                            
                        },
                    }
                },
                _ => {},
                Token::Identifier(_) => todo!(),
                Token::IntLiteral(_) => todo!(),
                Token::FloatLiteral(_) => todo!(),
                Token::StringLiteral(_) => todo!(),
                Token::AdditionOperator => todo!(),
                Token::SubtractionOperator => todo!(),
                Token::MultiplicationOperator => todo!(),
                Token::DivisionOperator => todo!(),
                Token::ModuloOperator => todo!(),
                Token::AssignmentOperator => todo!(),
                Token::Equal => todo!(),
                Token::Less => todo!(),
                Token::LessEqual => todo!(),
                Token::More => todo!(),
                Token::MoreEqual => todo!(),
                Token::LeftParen => todo!(),
                Token::RightParen => todo!(),
                Token::LeftBrace => todo!(),
                Token::RightBrace => todo!(),
                Token::Comma => todo!(),
                Token::Dot => todo!(),
                Token::SemiColon => todo!(),
                Token::EOF => todo!(),
            }
            scope_body.instructions.push(Box(node));
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
            return Node::Import(import {
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
        return Node::Import(import {
            import_name: if let Token::StringLiteral(s) = import_name {
                s
            } else {
                String::new()
            },
            alias: None,
        });
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
            }
        }
        let _ = self.eat(tokenizer::Token::LeftBrace);
        let function_body = self.parse_body();
        let _ = self.eat(tokenizer::Token::RightBrace);
        let _function_semi = self.eat(tokenizer::Token::SemiColon);
        return Node::Function(func {
            function_name: if let Token::Identifier(id) = func_name {
                id
            } else {
                String::new()
            },
            body:function_body,
            parameter:vec![],
            return_type
        });
    }
    fn parse_var_init(&mut self) -> Node{
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
        return Node::Variable(var{constant,name:var_name,var_type:data_type})
    }
    pub fn parse(&mut self) {
        let mut program = body {
            instructions: vec![],
        };
        while self.look_ahead != Token::EOF {
            let mut node:Node = match &self.look_ahead {
                Token::Keyword(keyword) => {
                    match keyword {
                        KeyWords::FUNC => {
                            return self.parse_function();
                        },
                        KeyWords::IMPORT => {
                            return self.parse_import();
                        },
                        KeyWords::AS => {
                            panic!("unexpected keyword \"as\"");
                        },
                        KeyWords::CONST => todo!(),
                        KeyWords::RETURN => todo!(),
                    }
                },
                _ => {},
                Token::Identifier(_) => todo!(),
                Token::IntLiteral(_) => todo!(),
                Token::FloatLiteral(_) => todo!(),
                Token::StringLiteral(_) => todo!(),
                Token::AdditionOperator => todo!(),
                Token::SubtractionOperator => todo!(),
                Token::MultiplicationOperator => todo!(),
                Token::DivisionOperator => todo!(),
                Token::ModuloOperator => todo!(),
                Token::AssignmentOperator => todo!(),
                Token::Equal => todo!(),
                Token::Less => todo!(),
                Token::LessEqual => todo!(),
                Token::More => todo!(),
                Token::MoreEqual => todo!(),
                Token::LeftParen => todo!(),
                Token::RightParen => todo!(),
                Token::LeftBrace => todo!(),
                Token::RightBrace => todo!(),
                Token::Comma => todo!(),
                Token::Dot => todo!(),
                Token::SemiColon => todo!(),
                Token::EOF => todo!(),
            }
            program.instructions.push(Box(node));
        }

        self.ast = Some(Box::new(self.parse_import()));
        println!("{:?}", self.ast);

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
}
