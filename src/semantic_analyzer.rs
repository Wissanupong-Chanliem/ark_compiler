use std::{borrow::BorrowMut, collections::HashMap, rc::Rc};
use crate::{arkparser::{AstNode, BinExp, Body, FuncDef, LiteralValue, Node, ParserError, Var}, symbol_table::{self, SymbolTable}, tokenizer::{DataType, Token}, CompilerError, ErrorPipeline, ErrorType};
use crate::tokenizer::TokenType;
use enum_map::{enum_map,EnumMap};

pub struct OperationValidator {
    allow_list:Vec<Operation>,
}

impl OperationValidator{
    pub fn add_allow(&mut self,left:DataType,right:DataType,operator:TokenType){
        self.allow_list.push(
            Operation::new(left, right, operator)
        )
    }
    pub fn validate(&self,left:DataType,right:DataType,operator:TokenType) -> bool {
        for rule in &self.allow_list {
            if rule.operation == operator && rule.left == left && rule.right == right {
                return true
            }
        }
        return false;
    }
    pub fn eval(&self,left:Node,right:Node,operator:TokenType){

    }
}

pub struct Operation {
    left:DataType,
    right:DataType,
    operation:TokenType
}
impl Operation {
    pub fn new(left:DataType,right:DataType,operator:TokenType) -> Self{
        Operation {
            left,
            right,
            operation:operator
        }
    }
}

pub struct SemanticAnalyzer<'a,'b> {
    ast:&'a Body,
    error_pipe:&'b ErrorPipeline,
    symbol_table:Rc<SymbolTable>,
    operation_validator:OperationValidator
}

impl<'a,'b> SemanticAnalyzer<'a,'b> {
    pub fn new(ast:&'a Body,error_pipe:&'b ErrorPipeline,table:Rc<SymbolTable>) -> SemanticAnalyzer<'a,'b> {
        SemanticAnalyzer {
            ast,
            error_pipe,
            symbol_table:table,
            operation_validator:OperationValidator {allow_list:vec![]}
        }
    }

    fn get_int_type(int:i64) -> Option<DataType> {
        if int <= i8::MAX.into() && int >= i8::MIN.into(){
            Some(DataType::I8)
        }
        else if int <= i16::MAX.into() && int >= i16::MIN.into(){
            Some(DataType::I16)
        }
        else if int <= i32::MAX.into() && int >= i32::MIN.into(){
            Some(DataType::I32)
        }
        else if int <= i64::MAX.into() && int >= i64::MIN.into(){
            Some(DataType::I64)
        }
        else{
            None
        }
    }

    fn get_float_type(float:f64) -> Option<DataType> {
        if float <= f32::MAX.into() && float >= f32::MIN.into(){
            Some(DataType::F32)
        }
        else if float <= f64::MAX.into() && float >= f64::MIN.into(){
            Some(DataType::F64)
        }
        else{
            None
        }
    }

    // fn type_castable(original_type:DataType,target_type:DataType) -> bool{
    //     let castable :&'static EnumMap<DataType,Vec<DataType>> = &enum_map! {
    //         DataType::Void => vec![],
    //         DataType::Char => vec![DataType::I32,DataType::I64],
    //         DataType::Boolean => vec![DataType::U8,DataType::U16,DataType::U32,DataType::U64,DataType::I8,DataType::I16,DataType::I32,DataType::I64],
    //         DataType::U8 => vec![DataType::U16,DataType::U32,DataType::U64],
    //         DataType::U16 => vec![DataType::U32,DataType::U64],
    //         DataType::U32 => vec![DataType::U64],
    //         DataType::U64 => vec![],
    //         DataType::I8 => vec![DataType::I16,DataType::I32,DataType::I64],
    //         DataType::I16 => vec![DataType::I32,DataType::I64],
    //         DataType::I32 => vec![DataType::I64],
    //         DataType::I64 => vec![],
    //         DataType::F32 => vec![DataType::F64],
    //         DataType::F64 => vec![]
    //     };
    //     return castable[original_type].contains(&target_type);
    //     // } HashMap<DataType,Vec<DataType>> = HashMap::from([
    //     //     (DataType::Char,vec![DataType::I32,DataType::I64]),
    //     //     (DataType::Boolean,vec![DataType::U8,DataType::U16,DataType::U32,DataType::U64,DataType::I8,DataType::I16,DataType::I32,DataType::I64])
    //     //     (DataType::)
    //     // ]);
    // }

    fn check_expression_type(&self, node:&AstNode<Node>,symbol_table:Rc<SymbolTable>) -> Option<DataType>{
        match &node.node {
            Node::Variable(v) => {
                match self.symbol_table.lookup_var(v.clone()){
                    Some((var,scope)) => {
                        let line = node.pos.0;
                        if let Some(line_declare) = var.line_declare {
                            if line < line_declare {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        format!("use of unclared variable '{}'",v).as_str(),
                                        node.pos,
                                        v.len() as u32,
                                    )
                                );
                                symbol_table.var_push_line_ref_at(scope, v.clone(), node.pos.0);
                                return None;
                            }
                            symbol_table.var_push_line_ref_at(scope, v.clone(), node.pos.0);
                            return Some(var.data_type.unwrap());
                        }
                        return None;
                    },
                    None => {
                        self.error_pipe.report_error(
                            CompilerError::new(
                                ErrorType::SemanticError,
                                format!("use of unclared variable '{}'",v).as_str(),
                                node.pos,
                                v.len() as u32,
                            )
                        );
                        return None;
                    }
                }
            }
            Node::Assignment(tk) => {
                let left_type = if let Node::Variable(v) = &tk.left.node {
                    match self.symbol_table.lookup_var(v.clone()){
                        Some((var,scope)) => {
                            let line = node.pos.0;
                            if let Some(line_declare) = var.line_declare {
                                if line < line_declare {
                                    self.error_pipe.report_error(
                                        CompilerError::new(
                                            ErrorType::SemanticError,
                                            format!("use of unclared variable '{}'",v).as_str(),
                                            node.pos,
                                            v.len() as u32,
                                        )
                                    );
                                    symbol_table.var_push_line_ref_at(scope, v.clone(), node.pos.0);
                                    return None;
                                }
                                symbol_table.var_push_line_ref_at(scope, v.clone(), node.pos.0);
                                var.data_type.unwrap()
                            }
                            else{
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        format!("use of unclared variable '{}'",v).as_str(),
                                        node.pos,
                                        v.len() as u32,
                                    )
                                );
                                symbol_table.var_push_line_ref_at(scope, v.clone(), node.pos.0);
                                return None;
                            }
                        },
                        None => {
                            self.error_pipe.report_error(
                                CompilerError::new(
                                    ErrorType::SemanticError,
                                    format!("use of unclared variable '{}'",v).as_str(),
                                    node.pos,
                                    v.len() as u32,
                                )
                            );
                            return None;
                        }
                    }
                    
                }
                else{
                    self.error_pipe.report_error(
                        CompilerError::new(
                            ErrorType::SemanticError,
                            format!("invalid left operand assignment opration").as_str(),
                            node.pos,
                            node.length,
                        )
                    );
                    return None;
                };
                match self.check_expression_type(&tk.right, symbol_table){
                    Some(right_type) => {
                        if left_type == right_type {
                            return Some(left_type);
                        }
                        return None;
                    },
                    None => {
                        return None;
                    }
                }
                
            },
            Node::Literal(literal) => {
                match literal {
                    LiteralValue::Int(i) => {
                        match Self::get_int_type(*i){
                            Some(t) => return Some(t),
                            None => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        format!("value is too large").as_str(),
                                        node.pos,
                                        node.length,
                                    )
                                );
                                return None;
                            }
                        }
                    },
                    LiteralValue::Float(f) => {
                        match Self::get_float_type(*f){
                            Some(t) => return Some(t),
                            None => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        format!("value is too large").as_str(),
                                        node.pos,
                                        node.length,
                                    )
                                );
                                return None;
                            }
                        }
                    },
                    LiteralValue::Str(_) => todo!(),
                }
            },
            Node::BinaryExpression(_) => todo!(),
            Node::FunctionCall(_) => todo!(),
            Node::MethodCall(_) => todo!(),
            Node::BooleanNot(_) => todo!(),
            Node::Tuple(_) => todo!(),
            Node::Range(_) => todo!(),
            _ => todo!()
        }
        //self.operation_validator.validate(exp.left.node, exp.right.node, exp.operator);
    }

    fn analyze_node(&self,node:&AstNode<Node>,symbol_table:Rc<SymbolTable>){
        match &node.node {
            Node::Body(bd) => {
                self.analyze_body(bd,symbol_table.insert_block_scope());
            },
            Node::Variable(v) => {
                match self.symbol_table.lookup_var(v.clone()){
                    Some((var,scope)) => {
                        let line = node.pos.0;
                        if let Some(line_declare) = var.line_declare {
                            if line < line_declare {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        format!("use of unclared variable '{}'",v).as_str(),
                                        node.pos,
                                        v.len() as u32,
                                    )
                                );
                            }
                        }
                        symbol_table.var_push_line_ref_at(scope, v.clone(), node.pos.0);
                        
                    },
                    None => {
                        self.error_pipe.report_error(
                            CompilerError::new(
                                ErrorType::SemanticError,
                                format!("use of unclared variable '{}'",v).as_str(),
                                node.pos,
                                v.len() as u32,
                            )
                        );
                    }
                }
            },
            Node::DeclareVar(v) => {
                symbol_table.insert_var(v.name.node.clone());
                symbol_table.update_var(
                    v.name.node.clone(),
                    Some(v.var_type.node.clone()),
                    Some(v.var_type.node.get_size_in_bytes()),
                    Some(0),
                    Some(v.name.pos.0)
                )
            },
            Node::Assignment(exp) => {
                let left = &exp.left.node;
            },
            Node::Literal(_) => todo!(),
            Node::BinaryExpression(_) => todo!(),
            Node::Function(_) => todo!(),
            Node::FunctionCall(_) => todo!(),
            Node::MethodCall(_) => todo!(),
            Node::Import(_) => todo!(),
            Node::Return(_) => todo!(),
            Node::Conditional(_) => todo!(),
            Node::For(_) => todo!(),
            Node::While(_) => todo!(),
            Node::BooleanNot(_) => todo!(),
            Node::Tuple(_) => todo!(),
            Node::Range(_) => todo!(),
            Node::ParserError(_) => todo!(),
        };
    }

    fn analyze_body(&self,bd:&Body,scope_symbol_table:Rc<SymbolTable>){
        for node in &bd.instructions{
            self.analyze_node(node,Rc::clone(&scope_symbol_table));
        }
    }

    pub fn analyze(&self) {
        for node in &self.ast.instructions{
            self.analyze_node(node,Rc::clone(&self.symbol_table));
        }
    }
    
    // fn check(node:&Node) -> Option<Vec<ParserError>> {
    
    // }
}
