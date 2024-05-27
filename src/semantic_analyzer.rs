use std::{any::Any, borrow::BorrowMut, collections::HashMap, mem::discriminant, rc::Rc, thread::scope};
use crate::{arkparser::{AstNode, BinExp, Body, FuncDef, LiteralValue, Node, ParserError, Var}, symbol_table::{self, Scope, SymbolTable}, tokenizer::{Array, DataType, Token}, CompilerError, ErrorPipeline, ErrorType};
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
    pub fn new(ast:&'a Body,error_pipe:&'b ErrorPipeline) -> SemanticAnalyzer<'a,'b> {
        SemanticAnalyzer {
            ast,
            error_pipe,
            symbol_table:Rc::new(SymbolTable::new(symbol_table::Scope::Global)),
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

    fn type_castable(original_type:&DataType,target_type:&DataType) -> bool{
        let castable: Vec<(DataType, Vec<DataType>)> = vec![
            (DataType::Void , vec![]),
            (DataType::Char , vec![DataType::I32,DataType::I64]),
            (DataType::Str(0) , vec![]),
            (DataType::Array(Array {length:0,data_type:Box::new(DataType::Void)}) , vec![]),
            (DataType::Boolean , vec![DataType::U8,DataType::U16,DataType::U32,DataType::U64,DataType::I8,DataType::I16,DataType::I32,DataType::I64]),
            (DataType::U8 , vec![DataType::U16,DataType::U32,DataType::U64]),
            (DataType::U16 , vec![DataType::U32,DataType::U64]),
            (DataType::U32 , vec![DataType::U64]),
            (DataType::U64 , vec![]),
            (DataType::I8 , vec![DataType::I16,DataType::I32,DataType::I64]),
            (DataType::I16 , vec![DataType::I32,DataType::I64]),
            (DataType::I32 , vec![DataType::I64]),
            (DataType::I64 , vec![]),
            (DataType::F32 , vec![DataType::F64]),
            (DataType::F64 , vec![])
        ];
        for (o_type, possible_types) in castable {
            if discriminant(&o_type) == discriminant(original_type) {
                return possible_types.contains(target_type);
                // for t in possible_types{
                //     if discriminant(&target_type) == discriminant(&t) {
                //         return true;
                //     }
                // }
                // break;
            } 
        }
        return false;
        // } HashMap<DataType,Vec<DataType>> = HashMap::from([
        //     (DataType::Char,vec![DataType::I32,DataType::I64]),
        //     (DataType::Boolean,vec![DataType::U8,DataType::U16,DataType::U32,DataType::U64,DataType::I8,DataType::I16,DataType::I32,DataType::I64])
        //     (DataType::)
        // ]);
    }

    fn type_coercion(&self,first_type:&DataType,second_type:&DataType) -> Option<DataType>{
        if Self::type_castable(first_type, second_type){
            return Some(second_type.clone());
        }
        if Self::type_castable( second_type,first_type){
            return Some(first_type.clone());
        }
        None
    }

    fn check_expression_type(&self, node:&AstNode<Node>,symbol_table:&Rc<SymbolTable>) -> Option<DataType>{
        let symbol_table = Rc::clone(symbol_table);
        match &node.node {
            Node::Variable(v) => {
                match symbol_table.lookup_var(v.clone()){
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
                let left_type = match &tk.left.node {
                    Node::DeclareVar(v) => {
                        symbol_table.insert_var(v.name.node.clone());
                        symbol_table.update_var(
                            v.name.node.clone(),
                            Some(v.var_type.node.clone()),
                            Some(v.var_type.node.get_size_in_bytes()),
                            Some(0),
                            Some(v.name.pos.0)
                        );
                        v.var_type.node.clone()
                    },
                    Node::Variable(v) => {
                        match symbol_table.lookup_var(v.clone()){
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
                    },
                    _ => {
                        self.error_pipe.report_error(
                            CompilerError::new(
                                ErrorType::SemanticError,
                                format!("left operand can't assign to").as_str(),
                                node.pos,
                                node.length,
                            )
                        );
                        return None;
                    }
                };
                match self.check_expression_type(&tk.right, &symbol_table){
                    Some(right_type) => {
                        if discriminant(&left_type) == discriminant(&right_type) || Self::type_castable(&right_type,&left_type) {
                            return Some(left_type);
                        }
                        self.error_pipe.report_error(
                            CompilerError::new(
                                ErrorType::SemanticError,
                                format!("expected '{}' found '{}'",left_type.to_string(),right_type.to_string()).as_str(),
                                node.pos,
                                node.length,
                            )
                        );
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
                    LiteralValue::Str(st) => {
                        return Some(DataType::Str(st.len() as u32))
                    },
                    LiteralValue::Bool(b) => {
                        return Some(DataType::Boolean);
                    }
                }
            },
            Node::BinaryExpression(exp) => {
                let left_type = match self.check_expression_type(&exp.left, &symbol_table){
                    Some(dt) => dt,
                    None => return None
                };
                let right_type = match self.check_expression_type(&exp.right, &symbol_table){
                    Some(right_type) => {
                        
                        right_type
                    },
                    None => {
                        return None;
                    }
                };
                
                let operand_type = if left_type != right_type {
                    match self.type_coercion(&left_type, &right_type) {
                        Some(result_type) => result_type,
                        None => {
                            self.error_pipe.report_error(
                                CompilerError::new(
                                    ErrorType::SemanticError,
                                    format!("operand have mismatched type '{}' and '{}'",left_type.to_string(),right_type.to_string()).as_str(),
                                    exp.operator.pos,
                                    exp.operator.length,
                                )
                            );
                            return None
                        },
                    }
                }
                else{
                    left_type
                };
                match exp.operator.node {
                    TokenType::AdditionOperator => {
                        match operand_type {
                            DataType::Array(_) => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '+' cannot be used on Array",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            DataType::Boolean => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '+' cannot be used on bool type",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            DataType::Char => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '+' cannot be used on char type, consider making it a str type",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            _ => {
                                return Some(operand_type);
                            }
                        }
                    }
                    TokenType::SubtractionOperator => {
                        match operand_type {
                            DataType::Array(_) => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '-' cannot be used on Array",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            DataType::Boolean => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '-' cannot be used on bool type",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            DataType::Char => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '-' cannot be used on char type",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            DataType::Str(_) => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '-' cannot be used on str type",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            _ => {
                                return Some(operand_type);
                            }
                        }
                    },
                    TokenType::MultiplicationOperator => {
                        match operand_type {
                            DataType::Array(_) => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '*' cannot be used on Array",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            DataType::Boolean => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '*' cannot be used on bool type",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            DataType::Char => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '*' cannot be used on char type",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            DataType::Str(_) => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '*' cannot be used on str type",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            _ => {
                                return Some(operand_type);
                            }
                        }
                    },
                    TokenType::DivisionOperator => {
                        match operand_type {
                            DataType::Array(_) => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '/' cannot be used on Array",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            DataType::Boolean => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '/' cannot be used on bool type",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            DataType::Char => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '/' cannot be used on char type",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            DataType::Str(_) => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '/' cannot be used on str type",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            _ => {
                                return Some(operand_type);
                            }
                        }
                    },
                    TokenType::ModuloOperator => {
                        match operand_type {
                            DataType::Array(_) => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '%' cannot be used on Array",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            DataType::Boolean => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '%' cannot be used on bool type",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            DataType::Char => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '%' cannot be used on char type",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            DataType::Str(_) => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '%' cannot be used on str type",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            _ => {
                                return Some(operand_type);
                            }
                        }
                    },
                    TokenType::Equal => {
                        match operand_type {
                            DataType::Array(_) => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '==' cannot be used on Array",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            _ => {
                                return Some(DataType::Boolean);
                            }
                        }
                    },
                    TokenType::Less => {
                        match operand_type {
                            DataType::Array(_) => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '<' cannot be used on Array",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            DataType::Boolean => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '<' cannot be used on bool type",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            DataType::Char => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '<' cannot be used on char type",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            DataType::Str(_) => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '<' cannot be used on str type",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            _ => {
                                return Some(DataType::Boolean);
                            }
                        }
                    },
                    TokenType::LessEqual => {
                        match operand_type {
                            DataType::Array(_) => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '<=' cannot be used on Array",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            DataType::Boolean => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '<=' cannot be used on bool type",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            DataType::Char => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '<=' cannot be used on char type",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            DataType::Str(_) => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '<=' cannot be used on str type",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            _ => {
                                return Some(DataType::Boolean);
                            }
                        }
                    },
                    TokenType::More => {
                        match operand_type {
                            DataType::Array(_) => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '>' cannot be used on Array",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            DataType::Boolean => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '>' cannot be used on bool type",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            DataType::Char => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '>' cannot be used on char type",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            DataType::Str(_) => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '>' cannot be used on str type",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            _ => {
                                return Some(DataType::Boolean);
                            }
                        }
                    },
                    TokenType::MoreEqual => {
                        match operand_type {
                            DataType::Array(_) => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '>=' cannot be used on Array",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            DataType::Boolean => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '>=' cannot be used on bool type",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            DataType::Char => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '>=' cannot be used on char type",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            DataType::Str(_) => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '>=' cannot be used on str type",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            },
                            _ => {
                                return Some(DataType::Boolean);
                            }
                        }
                    },
                    TokenType::And => {
                        match operand_type {
                            DataType::Boolean => {
                                return Some(operand_type);
                            },
                            _ => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '&&' can only be used on boolean expression",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            }
                        }
                    },
                    TokenType::Or => {
                        match operand_type {
                            DataType::Boolean => {
                                return Some(operand_type);
                            },
                            _ => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "operator '||' can only be used on boolean expression",
                                        exp.operator.pos,
                                        exp.operator.length,
                                    )
                                );
                                return None;
                            }
                        }
                    },
                    _ => return None
                }
            },
            Node::FunctionCall(called) => {
                let func_att = match symbol_table.lookup_func(called.function_name.node.clone()){
                    Some((calling_func,_)) => {
                        calling_func
                    },
                    None => {
                        self.error_pipe.report_error(
                            CompilerError::new(
                                ErrorType::SemanticError,
                                format!("use of undeclared function '{}'",called.function_name.node).as_str(),
                                called.function_name.pos,
                                called.function_name.length,
                            )
                        );
                        return None;
                    }
                };
                return Some(func_att.return_type);
            },
            Node::MethodCall(_) => todo!(),
            Node::BooleanNot(_) => todo!(),
            Node::Tuple(_) => todo!(),
            Node::Range(_) => todo!(),
            _ => {
                println!("{:#?}",node);
                todo!()
            }
        }
        //self.operation_validator.validate(exp.left.node, exp.right.node, exp.operator);
    }

    fn analyze_node(&self,node:&AstNode<Node>,symbol_table:Rc<SymbolTable>){
        match &node.node {
            Node::Body(bd) => {
                self.analyze_body(bd,symbol_table.insert_block_scope());
            },
            Node::Variable(v) => {
                match symbol_table.lookup_var(v.clone()){
                    Some((var,scope)) => {
                        let line = node.pos.0;
                        if let Some(line_declare) = var.line_declare {
                            if line < line_declare {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        format!("use of undeclared variable '{}'",v).as_str(),
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
            Node::Assignment(_) => {
                self.check_expression_type(&node, &symbol_table);
            },
            Node::Literal(_) => {
                self.check_expression_type(&node, &symbol_table);
            },
            Node::BinaryExpression(_) => {
                self.check_expression_type(&node, &symbol_table);
            },
            Node::FunctionCall(_) => {
                self.check_expression_type(&node, &symbol_table);
            },
            Node::MethodCall(_) => {
                self.check_expression_type(&node, &symbol_table);
            },
            Node::Function(func) => {
                let func_block = symbol_table.insert_func(func.function_name.node.clone());
                symbol_table.update_func(func.function_name.node.clone(), Some(func.return_type.node.clone()), Some(func.function_name.pos.0));
                for param in &func.parameters {
                    let param_name = &param.node.name.node;
                    let param_type = &param.node.var_type.node;
                    symbol_table.func_push_param(func.function_name.node.clone(), (param_type.clone(),param_name.clone()));
                    func_block.insert_var(param_name.clone());
                    func_block.update_var(
                        param_name.clone(),
                        Some(param_type.clone()),
                        Some(param_type.get_size_in_bytes()),
                        Some(1),
                        Some(param.pos.0)
                    )
                }
                self.analyze_body(&func.body, func_block);
            },
            Node::Import(imp) => {
                if symbol_table.scope != Scope::Global{
                    self.error_pipe.report_error(
                        CompilerError::new(
                            ErrorType::SemanticError,
                            "only top-level import is allowed",
                            node.pos,
                            node.length,
                        )
                    );
                    return;
                }
                match &imp.alias {
                    Some(alias) => {
                        match self.symbol_table.lookup_var(alias.node.clone()){
                            Some(_) => {
                                self.error_pipe.report_error(
                                    CompilerError::new(
                                        ErrorType::SemanticError,
                                        "import alias overide existing identifier",
                                        alias.pos,
                                        alias.length,
                                    )
                                );
                            },
                            None => {
                                symbol_table.insert_var(alias.node.clone());
                            },
                        }
                    },
                    None => {

                    }
                }
                
            },
            Node::Return(ret) => {
                let current_func = match symbol_table.get_current_func_info(){
                    Some(func_info) => func_info,
                    None => {
                        self.error_pipe.report_error(
                            CompilerError::new(
                                ErrorType::SemanticError,
                                "top-level return is not allow",
                                node.pos,
                                node.length,
                            )
                        );
                        return ()
                    }
                };
                
                let return_value_type = match ret {
                    Some(return_exp) => {
                        match self.check_expression_type(&return_exp, &symbol_table){
                            Some(dt) => dt,
                            None => return ()
                        }
                    },
                    None => DataType::Void,
                };
                if self.type_coercion(&current_func.return_type, &return_value_type).is_none() && current_func.return_type != return_value_type {
                    self.error_pipe.report_error(
                        CompilerError::new(
                            ErrorType::SemanticError,
                            format!(
                                "function '{}' expect return type '{}' found '{}'",
                                current_func.func_name,
                                current_func.return_type.to_string(),
                                return_value_type.to_string()
                            ).as_str(),
                            node.pos,
                            node.length,
                        )
                    );
                }
            },
            Node::For(f) => {
                let for_block = symbol_table.insert_block_scope();
                match &f.var{
                    Some(v) => {
                        match &v.node {
                            Node::Variable(var) => {
                                for_block.insert_var(var.clone());
                            },
                            _ => ()
                        }
                    },
                    None => ()
                }
                self.analyze_body(&f.body, for_block);
            },
            Node::While(w) => {
                let while_block = symbol_table.insert_block_scope();
                let condition_type = match self.check_expression_type(&w.condition, &symbol_table){
                    Some(t) => t,
                    None => return (),
                };
                if discriminant(&condition_type) != discriminant(&DataType::Boolean){
                    self.error_pipe.report_error(
                        CompilerError::new(
                            ErrorType::SemanticError,
                            format!("while loop expected boolean expression found '{}'",condition_type.to_string()).as_str(),
                            w.condition.pos,
                            w.condition.length,
                        )
                    );
                }
                self.analyze_body(&w.body, while_block);
            },
            Node::BooleanNot(exp) => {
                let exp_type = match self.check_expression_type(&exp.exp, &symbol_table){
                    Some(t) => t,
                    None => return (),
                };
                if discriminant(&exp_type) != discriminant(&DataType::Boolean){
                    self.error_pipe.report_error(
                        CompilerError::new(
                            ErrorType::SemanticError,
                            format!("cannot apply ! to '{}'",exp_type.to_string()).as_str(),
                            node.pos,
                            node.length,
                        )
                    );
                }
            },
            Node::Conditional(con) => {
                let if_block = symbol_table.insert_block_scope();
                let condition_type = match self.check_expression_type(&con.if_block.0,&symbol_table){
                    Some(dt) => dt,
                    None => {
                        return ();
                    }
                };
                if condition_type != DataType::Boolean && !Self::type_castable(&condition_type,&DataType::Boolean){
                    self.error_pipe.report_error(
                        CompilerError::new(
                            ErrorType::SemanticError,
                            format!("expected boolean expression in if statement found '{}'",condition_type.to_string()).as_str(),
                            con.if_block.0.pos,
                            con.if_block.0.length,
                        )
                    );
                }
                self.analyze_body(&con.if_block.1, if_block);
                for (condition,body) in &con.elif_block{
                    let elif_block = symbol_table.insert_block_scope();
                    let condition_type = match self.check_expression_type(&condition,&symbol_table){
                        Some(dt) => dt,
                        None => {
                            return ();
                        }
                    };
                    if condition_type != DataType::Boolean && !Self::type_castable(&condition_type,&DataType::Boolean){
                        self.error_pipe.report_error(
                            CompilerError::new(
                                ErrorType::SemanticError,
                                format!("expected boolean expression in if statement found '{}'",condition_type.to_string()).as_str(),
                                con.if_block.0.pos,
                                con.if_block.0.length,
                            )
                        );
                    }
                    self.analyze_body(body, elif_block);
                }
                match &con.else_block {
                    Some(body) => {
                        let else_block = symbol_table.insert_block_scope();
                        self.analyze_body(body, else_block);
                    },
                    None => return ()
                }
            },
            Node::Tuple(_) => todo!(),
            Node::Range(_) => todo!(),
            Node::ParserError(_) => (),
        };
    }

    fn analyze_body(&self,bd:&Body,scope_symbol_table:Rc<SymbolTable>){
        for node in &bd.instructions{
            self.analyze_node(node,Rc::clone(&scope_symbol_table));
        }
    }

    pub fn analyze(&self) -> Rc<SymbolTable>{
        for node in &self.ast.instructions{
            self.analyze_node(node,Rc::clone(&self.symbol_table));
        }
        return Rc::clone(&self.symbol_table)
    }
    
    // fn check(node:&Node) -> Option<Vec<ParserError>> {
    
    // }
}
