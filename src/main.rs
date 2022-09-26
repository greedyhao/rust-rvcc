use std::fs::File;
use std::io::prelude::*;
use std::iter::Peekable;
use std::str::Chars;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser)]
    expression: String,
}

fn iterchar2str(start_char: char, iter: &mut Peekable<Chars>) -> String {
    let mut num = start_char.to_string();
    while let Some(next_ch) = iter.peek() {
        if next_ch >= &'0' && next_ch <= &'9' {
            num.push(iter.next().unwrap());
        } else {
            break;
        }
    }
    return num.trim_start().to_string();
}

fn main_body(args: Args, asm_name: &str) -> Result<i32, i32> {
    let str1 = "  .global main\n";
    let str2 = "main:\n";
    let mut str3 = "".to_string();
    let str4 = "  ret\n";

    // 这里我们将算式分解为 num (op num) (op num)...的形式
    let mut iter = args.expression.chars().peekable();
    while let Some(ch) = iter.next() {
        match ch {
            // 为num则传入a0
            '0'..='9' => {
                let num = iterchar2str(ch, &mut iter);
                str3 += &format!("  li a0, {}\n", num);
            }

            // 为+则读取下一个num做加法
            // addi rd, rs1, imm 表示 rd = rs1 + imm
            // addi中imm为有符号立即数，所以加法表示为 rd = rs1 + imm
            '+' => {
                if let Some(ch) = iter.next() {
                    let num = iterchar2str(ch, &mut iter);
                    str3 += &format!("  addi a0, a0, {}\n", num);
                } else {
                    return Err(-1);
                }
            }

            // 为-则读取下一个num做减法
            // addi rd, rs1, imm 表示 rd = rs1 + imm
            // addi中imm为有符号立即数，所以减法表示为 rd = rs1 + (-imm)
            '-' => {
                if let Some(ch) = iter.next() {
                    let num = iterchar2str(ch, &mut iter);
                    str3 += &format!("  addi a0, a0, -{}\n", num);
                } else {
                    return Err(-1);
                }
            }

            // 如果是空格就忽略掉
            ' ' => {}

            _ => {
                return Err(-1);
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

    fn test_xxx_do(input: &str, expect: &str, asm_name: &str) {
        use std::process::Command;
        use std::str;

        let args = Args {
            expression: input.to_string(),
        };
        main_body(args, asm_name).unwrap();

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
        assert_eq!(result, expect);
    }

    #[test]
    fn test_001_0_return_integer() {
        let asm_name = "tmp_001_0";
        let input = 1.to_string();
        test_xxx_do(input.as_str(), input.as_str(), asm_name);
    }

    #[test]
    fn test_001_1_return_integer() {
        let asm_name = "tmp_001_1";
        let input = 123.to_string();
        test_xxx_do(input.as_str(), input.as_str(), asm_name);
    }

    #[test]
    fn test_002_0_plus_and_minus() {
        let asm_name = "tmp_002_0";
        let input = "1-2+3";
        let expect = 1 - 2 + 3;
        test_xxx_do(input, expect.to_string().as_str(), asm_name);
    }

    #[test]
    fn test_002_1_plus_and_minus() {
        let asm_name = "tmp_002_1";
        let input = "112-22+33";
        let expect = 112 - 22 + 33;
        test_xxx_do(input, expect.to_string().as_str(), asm_name);
    }
    
    #[test]
    fn test_003_0_space_characters() {
        let asm_name = "tmp_003_0";
        let input = " 12 + 34 - 5 ";
        let expect = 12 + 34 - 5;
        test_xxx_do(input, expect.to_string().as_str(), asm_name);
    }
}
