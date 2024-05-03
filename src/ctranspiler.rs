use std::fmt::format;
use std::str::FromStr;
use crate::tokenizer::Token;
use crate::arkparser::{LiteralValue,Body,Node};
fn gen_c_code(node:Node) -> String{
    let mut c_code:String = String::new();
    match node{
            Node::Body(bd) => {
                c_code += (gen_body(&bd.instructions)+ "\n").as_str() ;
            },
            Node::Variable(id) => {
                c_code += id.as_str();
            },
            Node::DeclareVar(var) => {
                if var.constant {
                    c_code += "const ";
                }
                c_code += (var.var_type.to_c_type_string() + " ").as_str();
                c_code += var.name.as_str();
            },
            Node::Assignment(exp) => {
                c_code += gen_c_code(*exp.left).as_str();
                c_code += " = ";
                c_code += gen_c_code(*exp.right).as_str();
            },
            Node::Literal(li) => {
                c_code += match li {
                    LiteralValue::Int(i) => i.to_string(),
                    LiteralValue::Float(f) => f.to_string(),
                    LiteralValue::Str(s) => format!(r#"{}"#,s),
                }.as_str();
            },
            Node::BinaryExpression(exp) => {
                c_code += "(";
                c_code += gen_c_code(*exp.left).as_str();
                c_code += match exp.operator {
                    Token::AdditionOperator => " + ",
                    Token::SubtractionOperator => " - ",
                    Token::MultiplicationOperator => " * ",
                    Token::DivisionOperator => " / ",
                    Token::ModuloOperator => " % ",
                    _ => ""
                };
                c_code += gen_c_code(*exp.right).as_str();
                c_code += ")";
                
            },
            Node::Function(func) => {
                c_code += (func.return_type.to_c_type_string() + " ").as_str();
                c_code += (func.function_name + "(").as_str();
                for i in 0..func.parameter.len(){
                    c_code += (func.parameter[i].var_type.to_c_type_string() + " ").as_str();
                    c_code += func.parameter[i].name.as_str();
                    if i < func.parameter.len()-1{
                        c_code += ",";
                    }
                    
                }
                c_code += "){\n";
                c_code += gen_body(&func.body.instructions).as_str();
                c_code += "}";
                
            },
            Node::FunctionCall(func) => {
                c_code += func.function_name.as_str();
                c_code += "(";
                for i in 0..func.arguments.len(){
                    c_code += (gen_c_code(func.arguments[i].clone())).as_str();
                    if i < func.arguments.len()-1{
                        c_code += ",";
                    }
                    
                }
                c_code += ")";
            },
            Node::MethodCall(method) => {
                c_code += method.method_name.as_str();
                c_code += "(";
                for i in 0..method.arguments.len(){
                    c_code += (gen_c_code(method.arguments[i].clone())).as_str();
                    if i < method.arguments.len()-1{
                        c_code += ",";
                    }
                    c_code += ")";
                }
            },
            Node::Return(val) => {
                c_code += "return ";
                c_code += match val{
                        Some(exp) => {
                            gen_c_code(*exp)
                        },
                        None => {
                            String::new()
                        },
                    }.as_str() 
            },
            Node::Import(_) => todo!(),
            Node::Error(_) => todo!(),
        };
    c_code
}

fn gen_body(code:&Vec<Node>) -> String {
    let mut c_code = String::new();
    for node in code{
        c_code += gen_c_code(node.clone()).as_str();
        match node{
            Node::Function(_func)=>{
                c_code += "\n";
            }
            _ => {
                c_code += ";\n";
            }
        }
        
    }
    c_code
}

pub fn gen_c_program(code:&Vec<Node>) -> String {
    let mut c_code = String::from_str("#include <stdio.h>\n#include <stdint.h>\n").unwrap();
    for node in code{
        c_code += gen_c_code(node.clone()).as_str();
        match node{
            Node::Function(_func)=>{
                c_code += "\n";
            }
            _ => {
                c_code += ";\n";
            }
        }
        
    }
    c_code
}