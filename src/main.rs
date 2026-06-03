use std::{
    fs::File,
    io::{self, Write},
    os::unix::fs::PermissionsExt,
    path::{self, Path},
};

fn parse_args(input: &str) -> Vec<String> {
    let mut args: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut quote_char: Option<char> = None;
    let mut escaped = false;

    //if \ is active then the immediate effect
    for ch in input.chars() {
        if escaped && quote_char == None {
            current.push(ch);
            escaped = false;
            continue;
        }

        match quote_char {
            // currently inside quotes
            Some(q) => {
                if escaped {
                    // inside double quotes, \ only escapes " and \
                    if q == '"' && (ch == '"' || ch == '\\') {
                        current.push(ch); // drop the backslash, push literal
                    } else {
                        current.push('\\'); // backslash was literal, keep it
                        current.push(ch); // push the character too
                    }
                    escaped = false;
                } else if ch == '\\' && q == '"' {
                    escaped = true; // only activate escape inside double quotes
                } else if ch == q {
                    quote_char = None; // closing quote
                } else {
                    current.push(ch); // literal character
                }
            }
            // not inside quotes
            None => match ch {
                '\'' | '"' => quote_char = Some(ch),
                '\\' => escaped = true,
                ' ' => {
                    if !current.is_empty() {
                        args.push(current.clone());
                        current.clear();
                    }
                }
                _ => current.push(ch),
            },
        }
    }

    if !current.is_empty() {
        args.push(current);
    }

    args
}

struct Redirect {
    stdout: Option<String>,
}

fn parse_redirect(input: &str) -> (String,Redirect) {

    if let Some(pos) = input.find("1>") {
        let cmd = input[..pos].trim().to_string();
        let file = input[pos+2..].trim().to_string();
        return (cmd,Redirect {stdout:Some(file)});
    }

    if let some(pos) = input.find(">") {
        let cmd = input[..pos].trim().to_string();
        let file = input[pos+2..].trim.to_string();
        return (cmd,Redirect {stdout:{Some(file)}})
    }

    (input.trim().to_string(),Redirect {stdout:None})
}





fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        command = command.trim().to_string();

        let (command,redirect) = parse_redirect(&command);
        let command=command.as_str();

        let stdout_file = redirect.stdout.as_ref().map(|path| {
            File::create(path).unwrap()
        });

        if command == "exit" {
            break;
        }

        if command.starts_with("echo ") {
            let parts = parse_args(&command[5..]);
            let output = parts.join(" ");

            match stdout file {
                Some(mut file) => writeln!(file,"{}",output).unwrap,
                None => println!("{}",output),
            }
        } else if command.starts_with("type ") {
            let cmd = command.split_whitespace().nth(1).unwrap();

            match cmd {
                "echo" | "exit" | "type" | "pwd" | "cd" => {
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
        } else if command.starts_with("pwd") {
            let current_folder = std::env::current_dir().unwrap();
            println!("{}", current_folder.display());
        } else if command.starts_with("cd") {
            let new_dir = command.split_whitespace().nth(1).unwrap();

            if new_dir == "~" {
                let home_dir = std::env::home_dir().unwrap();
                std::env::set_current_dir(home_dir).unwrap();
            } else if Path::new(new_dir).exists() {
                std::env::set_current_dir(new_dir).unwrap();
            } else {
                println!("{}: No such file or directory", new_dir)
            }
        } else {
            let parts = parse_args(&command);
            let program = &parts[0];
            let args = &parts[1..];

            let path_var = std::env::var("PATH").unwrap_or_default();

            let mut found_path = None;

            for dir in path_var.split(':') {
                let full_path = Path::new(dir).join(program);

                if full_path.exists() {
                    let metadata = std::fs::metadata(&full_path).unwrap();

                    if metadata.permissions().mode() & 0o111 != 0 {
                        found_path = Some(full_path);
                        break;
                    }
                }
            }

            if let Some(path) = found_path {
                let mut child = std::process::Command::new(program)
                    
                cmd.args(args);

                if let Some(file) = stdout_file {
                    cmd.stdout(file);
                }

                cmd.spawn().unwrap().wait().unwrap();
            }
            
    }
}
