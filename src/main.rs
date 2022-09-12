use std::fs::File;
use std::io::prelude::*;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser)]
    expression: String,
}

fn main_body(args: Args) {
    let str1 = "  .global main\n";
    let str2 = "main:\n";
    let str3 = format!("  li a0, {}\n", args.expression);
    let str4 = "  ret\n";

    let str = str1.to_string() + str2 + &str3 + str4;

    let mut file = File::create("tmp.s").unwrap();
    write!(file, "{}", str).unwrap();
}

fn main() {
    let args = Args::parse();
    main_body(args)
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
    fn test_000_return_integer() {
        use std::process::Command;
        use std::str;

        let test_integer = 1.to_string();
        let args = Args {
            expression: test_integer.clone(),
        };
        main_body(args);

        let test = Command::new("sh")
            .arg("test.sh")
            .output()
            .expect("failed to execute script");
        println!("{:?} {:?} {:?}", test.status, test.stdout, test.stderr);

        let stdout = [test.stdout[0]];
        let stdout = str::from_utf8(&stdout).unwrap();
        println!("{}", stdout);
        assert_eq!(stdout, test_integer);
    }
}
