use std::{
    fs::Metadata,
    io::{self, Write},
    os::unix::fs::PermissionsExt,
};

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
            let cmd = command.split_whitespace().nth(1).unwrap();

            match cmd {
                "echo" | "exit" | "type" => {
                    println!("{} is a shell builtin", cmd);
                }
                _ => {
                    println!("{}: not found", cmd);
                }
            }

            let path = match std::env::var("PATH") {
                Ok(p) => p,
                Err(_) => {
                    println!("PATH not found");
                    return;
                }
            }
            .split(':');

            for dir in path {
                let full_path = Path::new(dir).join(cmd);

                if full_path.exists() {
                    let metadata = std::fs::metadata(&full_path).unwrap();
                    let permissions = metadata.permissions();
                    if metadata.permissions().mode() & 0o111 != 0 {
                        println!("{} is {}", cmd, full_path.display());
                        break;
                    }
                }
            }
        } else {
            println!("{}: not found", command);
        }
    }
}
