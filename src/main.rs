use core::slice::Iter;
use std::fs::File;
use std::io::prelude::*;
use std::iter::{Enumerate, Peekable};
use std::str::Chars;

use clap::Parser;
use log::{trace, info};
use std::fmt::Debug;

#[derive(Debug)]
pub struct BinaryTree<T> {
    head: Link<T>,
}

type Link<T> = Option<Box<Node<T>>>;

#[derive(Debug)]
struct Node<T> {
    key: T,
    left: Link<T>,
    right: Link<T>,
}

impl<T> BinaryTree<T> {
    pub fn new(key: T) -> Self {
        let node = Box::new(Node {
            key,
            left: None,
            right: None,
        });
        let mut bt = BinaryTree { head: None };
        bt.head = Some(node);
        return bt;
    }

    pub fn to_left(&mut self, new_root_key: T, right: &mut Option<BinaryTree<T>>) {
        let mut new_binary = Box::new(Node {
            key: new_root_key,
            left: self.head.take(),
            right: None,
        });

        if let Some(r) = right {
            new_binary.right = r.head.take();
        }

        self.head = Some(new_binary);
    }
}

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

#[derive(Debug, Clone)]
struct Token {
    kind: TokenKind,
    value: String,
}

#[derive(Debug)]
enum NodeKind {
    Add,
    Sub,
    Mul,
    Div,
    Num,
}

#[derive(Debug)]
struct AstNode {
    kind: NodeKind,
    value: String,
}

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
        trace!("tokenize index={} ch={}", index, ch);
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

        trace!(
            "tokenize token_kind={:?} token_kind_old={:?}",
            token_kind, token_kind_old
        );

        if token_kind_old != token_kind {
            let token = Token {
                kind: token_kind_old,
                value: token_value.clone(),
            };
            if !token_value.is_empty() {
                trace!("tokenize tokens.push:{:?}", token);
                tokens.push(token);
            }
            token_kind_old = token_kind;
            token_value.clear();
        }

        if token_kind != TokenKind::Ignore {
            trace!("tokenize token_value.push:{}", ch);
            token_value.push(*ch);
        }

        // 如果最后一个是数字
        if *index == count && token_kind == TokenKind::Num {
            tokens.push(Token {
                kind: token_kind,
                value: token_value.clone(),
            });
        }
        iter.next();
    }

    return tokens;
}

// expr = mul ("+" mul | "-" mul)*
// mul = primary ("*" primary | "/" primary)*
// primary = "(" expr ")" | num
fn expr(tokens: &mut Peekable<Iter<Token>>) -> Option<BinaryTree<AstNode>> {
    let mut node = match mul(tokens) {
        Some(node) => node,
        None => return None,
    };

    loop {
        let token = match tokens.peek() {
            Some(t) => t.clone().clone(),
            None => return Some(node),
        };
        trace!("-- expr {:?}", token);

        match token.value.as_str() {
            "+" => {
                tokens.next();
                trace!("expr {:?} == {:?}", tokens, token);
                node.to_left(
                    AstNode {
                        kind: NodeKind::Add,
                        value: token.value,
                    },
                    &mut mul(tokens),
                );
                continue;
            }
            "-" => {
                tokens.next();
                trace!("expr {:?} == {:?}", tokens, token);
                node.to_left(
                    AstNode {
                        kind: NodeKind::Sub,
                        value: token.value,
                    },
                    &mut mul(tokens),
                );
                continue;
            }
            _ => {
                return Some(node);
            }
        }
    }
}

fn mul(tokens: &mut Peekable<Iter<Token>>) -> Option<BinaryTree<AstNode>> {
    let mut node = match primary(tokens) {
        Some(node) => node,
        None => return None,
    };

    loop {
        let token = match tokens.peek() {
            Some(t) => t.clone().clone(),
            None => return Some(node),
        };
        trace!("-- mul {:?}", token);

        match token.value.as_str() {
            "*" => {
                tokens.next();
                node.to_left(
                    AstNode {
                        kind: NodeKind::Mul,
                        value: token.value,
                    },
                    &mut primary(tokens),
                );
                continue;
            }
            "/" => {
                tokens.next();
                node.to_left(
                    AstNode {
                        kind: NodeKind::Div,
                        value: token.value,
                    },
                    &mut primary(tokens),
                );
                continue;
            }
            _ => {
                return Some(node);
            }
        }
    }
}

fn primary(tokens: &mut Peekable<Iter<Token>>) -> Option<BinaryTree<AstNode>> {
    let token = match tokens.next() {
        Some(t) => t,
        None => return None,
    };

    match token.kind {
        TokenKind::Punck => {
            // trace!("primary Punck {:?} == {:?}", tokens, token);
            // match token.value.as_str() {
            //     "(" => {
            //         // expr(tokens, asts);
            //     }
            //     _ => {}
            // }
        }
        TokenKind::Num => {
            // trace!("primary Num {:?} == {:?}", tokens, token);
            // 新建一个节点
            return Some(BinaryTree::new(AstNode {
                kind: NodeKind::Num,
                value: token.value.clone(),
            }));
        }
        TokenKind::Ignore => {
            // 正常不会有这个元素
            // error
        }
    }

    return None;
}

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

    let tokens = tokenize(&args.expression);
    info!("tokens={:?}", tokens);
    let mut aiter = tokens.iter().peekable();
    let ast = expr(&mut aiter);
    info!("{:?}", ast);

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
    env_logger::init();
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

    #[test]
    fn test_binary_tree() {
        // let mut bt = BinaryTree::new("1");
        // bt.to_left("+", &mut BinaryTree::new("2"));
        // bt.to_left("*", &mut BinaryTree { head: None });

        // println!("{:?}", bt);
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
