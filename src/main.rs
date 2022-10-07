use core::slice::IterMut;
use std::fs::File;
use std::io::prelude::*;
use std::iter::{Enumerate, Peekable};
use std::str::Chars;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser)]
    expression: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum TokenKind {
    Ignore,
    Punck,
    Num,
    // Eof,
}

#[derive(Debug)]
struct Token {
    kind: TokenKind,
    value: String,
}

// #[derive(Debug)]
// struct Ast {
//     prev: usize,
//     curr: usize,
//     next: usize,
//     value: String,
// }

fn iterchar2str(
    current_char: char,
    iter: &mut Peekable<Enumerate<Chars>>,
) -> Result<String, (String, usize, char)> {
    let mut num = current_char.to_string();
    let mut check_operator = false;

    if !(current_char >= '0' && current_char <= '9') {
        num = String::new(); // 当前字符非数字，清空
        check_operator = true;
    }

    while let Some(&(index, next_ch)) = iter.peek() {
        match next_ch {
            '0'..='9' => {
                num.push(iter.next().unwrap().1);
                check_operator = false;
            }
            ' ' => {
                iter.next().unwrap();
            }
            _ => {
                if check_operator {
                    match next_ch {
                        '+' | '-' | '*' | '/' => {
                            return Err(("expect a number".to_string(), index, next_ch));
                        }
                        'a'..='z' | 'A'..='Z' => {
                            return Err(("invalid token".to_string(), index, next_ch));
                        }
                        _ => {}
                    }
                }
                break;
            }
        }
    }
    return Ok(num.trim_start().to_string());
}

fn tokenize(strings: &str) -> Vec<Token> {
    let count = strings.len() - 1;
    let mut iter = strings.chars().enumerate().peekable();

    let mut tokens: Vec<Token> = Vec::new();
    let mut token_kind = TokenKind::Ignore;
    let mut token_kind_old = TokenKind::Ignore;
    let mut token_value = String::new();

    while let Some((index, ch)) = iter.peek() {
        match *ch {
            '0'..='9' => {
                token_kind = TokenKind::Num;
            }
            _ => match ch {
                '+' | '-' | '*' | '/' | '(' | ')' => {
                    token_kind = TokenKind::Punck;
                }
                _ => token_kind = TokenKind::Ignore,
            },
        }

        if token_kind_old != token_kind || *index == count {
            let token = Token {
                kind: token_kind_old,
                value: token_value.clone(),
            };
            if !token_value.is_empty() {
                tokens.push(token);
            }
            token_kind_old = token_kind;
            token_value.clear();
        }

        if token_kind != TokenKind::Ignore {
            token_value.push(*ch);
        }
        iter.next();
    }

    return tokens;
}

// // expr = mul ("+" mul | "-" mul)*
// // mul = primary ("*" primary | "/" primary)*
// // primary = "(" expr ")" | num
// fn expr(tokens: &mut IterMut<Token>, asts: &mut Vec<Ast>) {
//     mul(tokens, asts);
//     let value = tokens.next();
//     let ast = Ast {
//         prev: 1,
//         curr: 1,
//         next: 2,
//     };
//     asts.push(ast);
//     println!("expr -- {:?} {:?}", tokens, asts);
// }

// fn mul(tokens: &mut IterMut<Token>, asts: &mut Vec<Ast>) {
//     primary(tokens, asts);
//     let ast = Ast {
//         prev: 2,
//         curr: 1,
//         next: 2,
//     };
//     asts.push(ast);
//     println!("mul -- {:?} {:?}", tokens.next(), asts);
// }

// fn primary(tokens: &mut IterMut<Token>, asts: &mut Vec<Ast>) {
//     let token = tokens.next();
//     let token = match token {
//         Some(t) => t,
//         None => return,
//     };

//     match token.value.as_str() {
//         "(" => {
//             expr(tokens, asts);
//         }
//         _ => {}
//     }

//     let ast = Ast {
//         prev: 3,
//         curr: 1,
//         next: 2,
//     };
//     asts.push(ast);
//     println!("primary -- {:?} {:?}", tokens, asts);
// }

fn main_body(args: Args, asm_name: &str) -> Result<i32, String> {
    let str1 = "  .global main\n";
    let str2 = "main:\n";
    let mut str3 = String::new();
    let str4 = "  ret\n";

    let mut str3_format = "";
    let str3_num = "  li a0, ";

    // addi rd, rs1, imm 表示 rd = rs1 + imm
    // addi中imm为有符号立即数，所以加法表示为 rd = rs1 + imm
    let str3_plus = "  addi a0, a0, ";

    // addi rd, rs1, imm 表示 rd = rs1 + imm
    // addi中imm为有符号立即数，所以减法表示为 rd = rs1 + (-imm)
    let str3_minus = "  addi a0, a0, -";

    // 这里我们将算式分解为 num (op num) (op num)...的形式
    let mut iter = args.expression.chars().enumerate().peekable();

    // let mut tokens = tokenize(&args.expression);
    // println!("tokens={:?}", tokens);
    // let mut ast: Vec<Ast> = Vec::new();
    // let mut aiter = tokens.iter_mut();
    // expr(&mut aiter, &mut ast);

    while let Some((_, ch)) = iter.next() {
        match ch {
            // 为num则传入a0
            '0'..='9' => {
                str3_format = str3_num;
            }

            // 为+则读取下一个num做加法
            '+' => {
                str3_format = str3_plus;
            }

            // 为-则读取下一个num做减法
            '-' => {
                str3_format = str3_minus;
            }

            // 如果是空格就忽略掉
            ' ' => {}

            _ => {
                return Err("undefined".to_string());
            }
        }

        if ch != ' ' {
            let res = iterchar2str(ch, &mut iter);
            match res {
                Ok(num) => str3 += &format!("{}{}\n", str3_format, num),
                Err((code, index, _)) => {
                    // 错误打印信息
                    let mut info = String::new();
                    info += &format!("{}\n", args.expression);
                    info += &format!("{: <1$}", "", index);
                    info += &format!("^ {}", code);
                    return Err(info);
                }
            }
        }
    }

    let str = str1.to_string() + str2 + &str3 + str4;

    let asm_name = asm_name.to_string() + ".s";
    let mut file = File::create(asm_name).unwrap();
    write!(file, "{}", str).unwrap();
    Ok(0)
}

fn main() {
    let args = Args::parse();
    main_body(args, "tmp.s").unwrap();
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[ignore]
    fn test_run_clean() {
        use std::process::Command;
        Command::new("sh").arg("clean.sh").output().unwrap();
    }

    fn test_xxx_do(input: &str, asm_name: &str) -> Result<String, String> {
        use std::process::Command;
        use std::str;

        let args = Args {
            expression: input.to_string(),
        };

        main_body(args, asm_name)?;

        let test = Command::new("sh")
            .arg("test.sh")
            .arg(asm_name)
            .output()
            .expect("failed to execute script");

        let stdout = str::from_utf8(&test.stdout).unwrap();
        // let stderr = str::from_utf8(&test.stderr).unwrap();
        // println!(
        //     "sh test.sh => {:?} {:?} {:?}",
        //     test.status,
        //     stdout,
        //     stderr,
        // );
        let tmp = stdout.to_string();
        let result: Vec<&str> = tmp.split('\n').collect();
        let result = result[0];

        return Ok(result.to_string());
    }

    #[test]
    fn test_001_0_return_integer() {
        let asm_name = "tmp_001_0";
        let input = 1.to_string();
        assert_eq!(test_xxx_do(input.as_str(), asm_name).unwrap(), input);
    }

    #[test]
    fn test_001_1_return_integer() {
        let asm_name = "tmp_001_1";
        let input = 123.to_string();
        assert_eq!(test_xxx_do(input.as_str(), asm_name).unwrap(), input);
    }

    #[test]
    fn test_002_0_plus_and_minus() {
        let asm_name = "tmp_002_0";
        let input = "1-2+3";
        let expect = 1 - 2 + 3;
        assert_eq!(test_xxx_do(input, asm_name).unwrap(), expect.to_string());
    }

    #[test]
    fn test_002_1_plus_and_minus() {
        let asm_name = "tmp_002_1";
        let input = "112-22+33";
        let expect = 112 - 22 + 33;
        assert_eq!(test_xxx_do(input, asm_name).unwrap(), expect.to_string());
    }

    #[test]
    fn test_003_0_space_characters() {
        let asm_name = "tmp_003_0";
        let input = " 12 + 34 - 5 ";
        let expect = 12 + 34 - 5;
        assert_eq!(test_xxx_do(input, asm_name).unwrap(), expect.to_string());
    }

    #[test]
    fn test_004_0_space_characters() {
        let asm_name = "tmp_004_0";
        let input = "1+s";
        assert_eq!(
            test_xxx_do(input, asm_name),
            Err("1+s\n  ^ invalid token".to_string())
        );
    }

    #[test]
    fn test_004_1_space_characters() {
        let asm_name = "tmp_004_1";
        let input = "1++1";
        assert_eq!(
            test_xxx_do(input, asm_name),
            Err("1++1\n  ^ expect a number".to_string())
        );
    }
}
