use std::{
    //fs::Metadata,
    io::{self, Write},
    os::unix::fs::PermissionsExt,
    panic::Full,
    path::Path,
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
            let parts = command.split_whitespace().unwrap();
            let program = parts[0];
            let args = &parts[1..];

            let path_var = std::env::var("PATH").unwrap_or_default();

            let mut found_path = None;

            for dir in path_var.split(':') {
                let full_path = Path::new(dir).join(cmd);

                if full_path.exists() {
                    let metadata = std::fs::metadata(&full_path).unwrap();

                    if metadata.permissions().mode() & 0o111 != 0 {
                        found_path = Some(full_path);
                        break;
                    }
                }
            }

            if let Some(path) = found_path {
                let mut child = std::process::Command::new(path).args(args).spawn().unwrap();
                child.wait().unwrap();
            } else {
                println!("{}: not found", program);
            }
        }
    }
}
