use std::vec;

pub struct lexer_output{
    tokens:Vec<Token>
}
pub struct Token<'a>{
    token_type:token_type,
    val:&'a str,
}

enum token_type{
    IntLiteral(u32),
    FloatLiteral(f64),
    StringLiteral,
    WhiteSpace,
    Operator,
    LeftParen,
    RightParen,
    EOF
}

const KEYWORDS:[&str;4] = [
    "let",
    "func",
    "use",
    "const",
];

fn lexical_analysis(source:&str)->lexer_output{
    let current_token:String = String::new();
    let index = 0;
    let mut tokens: Vec<Token> = vec![];
    loop{
        let ch = 
        match source.chars().nth(index){
            Some(ch)=>{
                if ch == '(' {
                    tokens.push(Token { token_type: token_type::LeftParen, val: "(" })
                }
                else if ch == ')' {
                    tokens.push(Token { token_type: token_type::RightParen, val: ")" })
                }
                else if ch == ' ' {
                    tokens.push(Token { token_type: token_type::WhiteSpace, val: " " })
                }
            },
            None=>{
                tokens.push(Token { token_type: token_type::EOF, val:"" })
            }
        };
    };


}

