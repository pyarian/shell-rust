#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        //wait for user input

        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        command = command.trim().to_string();

        //checking exit

        if command == "exit" {
            break;
        }

        if command.starts_with("type") {
            let c = command.split_whitespace().nth(1).unwrap();

            match c {
                "echo" | "exit" | "type" => {
                    println!("{} is a shell builtin", c);
                }
                _ => {
                    println!("{}: command not found ", c.trim());
                }
            }
        } else {
            println!("{}: not found", c.trim());
        }
    }
}
