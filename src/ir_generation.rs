use std::fmt::format;
use std::rc::Rc;
use std::str::FromStr;
use crate::symbol_table::{self, SymbolTable, VarAttribute};
use crate::tokenizer::{Token, TokenType};
use crate::arkparser::{AstNode, Body, LiteralValue, Node};

pub struct IRGenerator {
    symbol_table:Rc<SymbolTable>,
    code:String,
    //ir_table:IRBlock
}

impl IRGenerator{
    pub fn new(symbol_table:Rc<SymbolTable>) -> Self {
        return IRGenerator { symbol_table, code: String::new() }
    }
    pub fn get_intermediate_representation(&self,ast:&Body) -> String{
        let mut ir_code = String::new();
        for instruction in &ast.instructions {
            ir_code += Self::gen_ir(&instruction,self.symbol_table.clone()).as_str();
        }
        ir_code
    }

    fn gen_ir(node:&AstNode<Node>,symbol_table:Rc<SymbolTable>) -> String {
        let mut code = String::new();
        match &node.node {
            Node::Body(_) => {

            },
            Node::Variable(v) => {
                code += (v.clone() + symbol_table.get_var_version(v.clone()).to_string().as_str()).as_str()
            },
            Node::DeclareVar(var) => {
                code += (var.name.node.clone() + symbol_table.get_var_version(var.name.node.clone()).to_string().as_str()).as_str()
            },
            Node::Assignment(exp) => {
                let right_expression = Self::gen_ir(&exp.right, symbol_table.clone());
                let right = if right_expression.starts_with("tac_temp"){
                    code += right_expression.as_str();
                    "tac_temp".to_string() + &symbol_table.get_var_version(String::from("tac_temp")).to_string()
                }
                else{
                    right_expression
                };
                match &exp.left.node {
                    Node::Variable(v) => {
                        //let current_version = symbol_table.get_var_version(v.clone()).to_string().as_str();
                        code += format!("{}{} = {}\n",v.clone(),symbol_table.consume_var_version(v.clone()),right).as_str()

                    },
                    Node::DeclareVar(var) => {
                        //let current_version = symbol_table.get_var_version(var.name.node.clone()).to_string().as_str();
                        code += format!("{}{} = {}\n",var.name.node.clone(),symbol_table.consume_var_version(var.name.node.clone()),right).as_str();
                    },
                    _ => {}
                }
            },
            Node::Literal(l) => {
                match l {
                    LiteralValue::Int(i) => {
                        code += &i.to_string();
                    },
                    LiteralValue::Float(f) => {
                        code += &f.to_string();
                    },
                    LiteralValue::Str(s) => {
                        code += s;
                    },
                    LiteralValue::Bool(b) => {
                        code += &b.to_string();
                    },
                }
            },
            Node::BinaryExpression(binexp) => {
                let left_expression = IRGenerator::gen_ir(&binexp.left, symbol_table.clone());
                let left = if left_expression.starts_with("tac_temp"){
                    code = left_expression;
                    "tac_temp".to_string() + &symbol_table.get_var_version(String::from("tac_temp")).to_string()
                }
                else {
                    left_expression
                };
                let right_expression = IRGenerator::gen_ir(&binexp.right, symbol_table.clone());
                let right = if right_expression.starts_with("tac_temp"){
                    code += right_expression.as_str();
                    "tac_temp".to_string() + &symbol_table.get_var_version(String::from("tac_temp")).to_string()
                }
                else {
                    right_expression
                };
                if symbol_table.lookup_var(String::from("tac_temp")).is_none(){
                    symbol_table.insert_var(String::from("tac_temp"));
                }
                let op = match &binexp.operator.node {
                    TokenType::AdditionOperator => "add",
                    TokenType::SubtractionOperator => "sub",
                    TokenType::MultiplicationOperator => "mul",
                    TokenType::DivisionOperator => "div",  // Valid division operator
                    TokenType::ModuloOperator => "mod",  // Valid modulo operator
                    TokenType::Equal => "equ",
                    TokenType::Less => "l",
                    TokenType::LessEqual => "le",
                    TokenType::More => "m",
                    TokenType::MoreEqual => "me",
                    TokenType::And => "and",
                    TokenType::Or => "or",
                    TokenType::Not => "not",
                    _ => {""}
                };
                code += format!("tac_temp{} = {} {}, {}\n",symbol_table.consume_var_version(String::from("tac_temp")),op, left,right).as_str();
            },
            Node::Function(func) => {
                code += format!("@defined {} {}({}):\n",func.return_type.node.to_string(),func.function_name.node.clone(),func.parameters.iter().map(|p| format!("{} {}",p.node.var_type.node.to_string(),p.node.name.node)).collect::<Vec<String>>().join(", ")).as_str();
                let func_sym = symbol_table.lookup_func(func.function_name.node.clone()).unwrap().0.func_table.clone();
                for instruction in &func.body.instructions{
                    code += (Self::gen_ir(instruction, func_sym.clone()).lines().map(|i| "    ".to_string() + i).collect::<Vec<String>>().join("\n") + "\n").as_str();
                }
            },
            Node::FunctionCall(fun) => {
                let mut arguments:Vec<String> = vec![];
                for arg in &fun.arguments{
                    let argument_expression = Self::gen_ir(arg, symbol_table.clone());
                    let argument = if argument_expression.starts_with("tac_temp"){
                        code += argument_expression.as_str();
                        
                        "tac_temp".to_string() + &symbol_table.get_var_version(String::from("tac_temp")).to_string()
                    }
                    else {
                        argument_expression
                    };
                    arguments.push(argument);
                }
                
                if symbol_table.lookup_var(String::from("tac_temp")).is_none(){
                    symbol_table.insert_var(String::from("tac_temp"));
                };
                code += format!("{}{} = call {} {}\n","tac_temp",symbol_table.consume_var_version(String::from("tac_temp")),fun.function_name.node,arguments.join(", ")).as_str()
            },
            Node::MethodCall(_) => todo!(),
            Node::Import(_) => todo!(),
            Node::Return(exp) => {
                match exp {
                    Some(ex) => {
                        let ret_expression = IRGenerator::gen_ir(&ex, symbol_table.clone());
                        let ret_value = if ret_expression.starts_with("tac_temp"){
                            code = ret_expression;
                            "tac_temp".to_string() + &symbol_table.get_var_version(String::from("tac_temp")).to_string()
                        }
                        else {
                            ret_expression
                        };
                        code += format!("ret {}",ret_value).as_str();
                    },
                    None => {
                        code += "ret $void";
                    }
                };
            },
            Node::Conditional(_) => todo!(),
            Node::For(_) => todo!(),
            Node::While(_) => todo!(),
            Node::BooleanNot(_) => todo!(),
            Node::Tuple(_) => todo!(),
            Node::Range(_) => todo!(),
            Node::ParserError(_) => todo!(),
        }
        code
    }

}

// enum IrStatement{
//     Assignment((String,String)),
//     Expression,
//     Label((String,Box<IRBlock>)),
//     Goto,
// }

// enum Atom {
//     Literal
// }

// struct AssignmentStatement{
//     result:VarAttribute,
//     right:VarAttribute

// }

// struct IRBlock {
//     rows:Vec<IrStatement>
// }

// impl IRBlock{
//     pub fn new() -> Self {
//         return IRBlock { rows:vec![] }
//     }
// }