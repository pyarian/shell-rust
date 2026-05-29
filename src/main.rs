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

        if command.starts_with("echo") || command.starts_with("exit") || command.starts_with("type")
        {
            println!("{} is a shell builtin", command.trim());
        } else {
            println!("{}: command not found ", command.trim());
        }
    }
}
