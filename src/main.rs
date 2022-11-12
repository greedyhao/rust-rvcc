use core::slice::Iter;
use std::fs::File;
use std::io::prelude::*;
use std::iter::Peekable;

use clap::Parser;
use core::mem;
use log::{info, trace};
use std::fmt::Debug;

mod binary_tree;
use binary_tree::*;

impl BinaryTreeIO<AstNode> for BinaryTree<AstNode> {
    fn new_binary_without_right(
        root: AstNode,
        left: &mut BinaryTree<AstNode>,
    ) -> BinaryTree<AstNode> {
        let mut bt = BinaryTree::new(root);
        bt.set_left(Some(Box::new(mem::take(left))));
        return bt;
    }
}

impl Default for BinaryTree<AstNode> {
    fn default() -> Self {
        BinaryTree::new(AstNode {
            kind: NodeKind::Num,
            value: String::new(),
        })
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

#[derive(Debug, PartialEq)]
enum NodeKind {
    Add,
    Sub,
    Mul,
    Div,
    Num,
}

#[derive(Debug)]
pub struct AstNode {
    kind: NodeKind,
    value: String,
}

impl Default for AstNode {
    fn default() -> Self {
        AstNode {
            kind: NodeKind::Num,
            value: String::new(),
        }
    }
}

fn tokenize(strings: &str) -> Vec<Token> {
    let count = strings.len() - 1;
    let mut iter = strings.chars().enumerate().peekable();

    let mut tokens: Vec<Token> = Vec::new();
    let mut token_kind;
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
            token_kind,
            token_kind_old
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

                let mut bt = BinaryTree::new_binary_without_right(
                    AstNode {
                        kind: NodeKind::Add,
                        value: token.value,
                    },
                    &mut node,
                );

                let right = mul(tokens);
                if right.is_some() {
                    bt.set_right(Some(Box::new(mem::take(&mut right.unwrap()))));
                }
                node = bt;
                continue;
            }
            "-" => {
                tokens.next();
                trace!("expr {:?} == {:?}", tokens, token);

                let mut bt = BinaryTree::new_binary_without_right(
                    AstNode {
                        kind: NodeKind::Sub,
                        value: token.value,
                    },
                    &mut node,
                );

                let right = mul(tokens);
                if right.is_some() {
                    bt.set_right(Some(Box::new(mem::take(&mut right.unwrap()))));
                }
                node = bt;
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

                let mut bt = BinaryTree::new_binary_without_right(
                    AstNode {
                        kind: NodeKind::Mul,
                        value: token.value,
                    },
                    &mut node,
                );

                let right = primary(tokens);
                if right.is_some() {
                    bt.set_right(Some(Box::new(mem::take(&mut right.unwrap()))));
                }
                node = bt;
                continue;
            }
            "/" => {
                tokens.next();

                let mut bt = BinaryTree::new_binary_without_right(
                    AstNode {
                        kind: NodeKind::Div,
                        value: token.value,
                    },
                    &mut node,
                );

                let right = primary(tokens);
                if right.is_some() {
                    bt.set_right(Some(Box::new(mem::take(&mut right.unwrap()))));
                }
                node = bt;
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
            trace!("primary Punck {:?} == {:?}", tokens, token);
            // match token.value.as_str() {
            //     "(" => {
            //         // expr(tokens, asts);
            //     }
            //     _ => {}
            // }
        }
        TokenKind::Num => {
            trace!("primary Num {:?} == {:?}", tokens, token);
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

fn push(asm: &mut String, depth: &mut isize) {
    asm.push_str("  addi sp, sp, -8\n");
    asm.push_str("  sd a0, 0(sp)\n");
    *depth += 1;
}

fn pop(asm: &mut String, depth: &mut isize, reg: &str) {
    asm.push_str(&format!("  ld {}, 0(sp)\n", reg));
    asm.push_str("  addi sp, sp, 8\n");
    *depth -= 1;
}

fn get_expr(ast: Option<&BinaryTree<AstNode>>, asm: &mut String, depth: &mut isize) {
    trace!("get_expr ast {:?}", ast);
    let root = ast.as_ref().unwrap().get_root();
    if root.kind == NodeKind::Num {
        asm.push_str(&format!("  li a0, {}\n", root.value));
        trace!("get_expr num {:?}", root.value);
        return;
    }

    get_expr(ast.as_ref().unwrap().get_right(), asm, depth);
    push(asm, depth);
    get_expr(ast.as_ref().unwrap().get_left(), asm, depth);
    pop(asm, depth, "a1");

    trace!("get_expr {:?} {:?}", root.kind, root.value);
    match root.kind {
        NodeKind::Add => asm.push_str("  add a0, a0, a1\n"),
        NodeKind::Sub => asm.push_str("  sub a0, a0, a1\n"),
        NodeKind::Mul => asm.push_str("  mul a0, a0, a1\n"),
        NodeKind::Div => asm.push_str("  div a0, a0, a1\n"),
        _ => {
            // error
        }
    }
}

fn main_body(args: Args, asm_name: &str) -> Result<i32, String> {
    let str1 = "  .global main\n";
    let str2 = "main:\n";
    let mut str3 = String::new();
    let str4 = "  ret\n";

    let tokens = tokenize(&args.expression);
    info!("tokens={:?}", tokens);
    let mut aiter = tokens.iter().peekable();
    let ast = expr(&mut aiter);
    info!("{:?}", ast);

    let mut depth = 0;
    get_expr(ast.as_ref(), &mut str3, &mut depth);

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
    fn test_bt() {
        let mut bt1 = BinaryTree::new(AstNode {
            kind: NodeKind::Num,
            value: "1".to_string(),
        });
        bt1.set_left(Some(Box::new(BinaryTree::new(AstNode {
            kind: NodeKind::Num,
            value: "0".to_string(),
        }))));
        let bt2 = BinaryTree::new_binary_without_right(
            AstNode {
                kind: NodeKind::Add,
                value: String::new(),
            },
            &mut bt1,
        );

        let left = bt2.get_left();
        let lleft = left.unwrap().get_left();

        println!("{:?}", left);
        println!("{:?}", lleft);
    }

    fn test_xxx_do(input: &str, asm_name: &str) -> Result<String, String> {
        use std::process::Command;
        use std::str;
        let _ = env_logger::builder().is_test(true).try_init();

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
        let stderr = str::from_utf8(&test.stderr).unwrap();
        info!("sh test.sh => {:?} {:?} {:?}", test.status, stdout, stderr,);
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
