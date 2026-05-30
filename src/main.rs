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
                    let path_var = match std::env::var("PATH") {
                        Ok(p) => p,
                        Err(_) => {
                            println!("PATH not found");
                            continue;
                        }
                    };

                    let mut found = false;

                    for dir in path_var.split(':') {
                        let full_path = Path::new(dir).join(cmd);

                        if full_path.exists() {
                            let metadata = std::fs::metadata(&full_path).unwrap();

                            if metadata.permissions().mode() & 0o111 != 0 {
                                println!("{} is {}", cmd, full_path.display());
                                found = true;
                                break;
                            }
                        }
                    }

                    if !found {
                        println!("{}: not found", cmd);
                    }
                }
            }
        } else {
            println!("{}: not found", command);
        }
    }
}
