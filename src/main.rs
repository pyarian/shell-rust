use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        command = command.trim().to_string();

        if command == "exit" {
            break;
        }

        if command.starts_with("echo ") {
            println!("{}", &command[5..]);
        } else if command.starts_with("type ") {
            let c = command.split_whitespace().nth(1).unwrap();

            match c {
                "echo" | "exit" | "type" => {
                    println!("{} is a shell builtin", c);
                }
                _ => {
                    println!("{}: not found", c);
                }
            }
        } else {
            println!("{}: not found", command);
        }
    }
}
