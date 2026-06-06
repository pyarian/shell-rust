use std::{
    fs::File,
    io::{self, Write},
    os::unix::fs::PermissionsExt,
    path::Path,
};

fn parse_args(input: &str) -> Vec<String> {
    let mut args: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut quote_char: Option<char> = None;
    let mut escaped = false;

    for ch in input.chars() {
        if escaped && quote_char == None {
            current.push(ch);
            escaped = false;
            continue;
        }

        match quote_char {
            Some(q) => {
                if escaped {
                    if q == '"' && (ch == '"' || ch == '\\') {
                        current.push(ch);
                    } else {
                        current.push('\\');
                        current.push(ch);
                    }
                    escaped = false;
                } else if ch == '\\' && q == '"' {
                    escaped = true;
                } else if ch == q {
                    quote_char = None;
                } else {
                    current.push(ch);
                }
            }
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

enum RedirectMode {
    Overwrite,
    Append,
}

struct Redirect {
    stdout: Option<(String, RedirectMode)>,
    stderr: Option<(String, RedirectMode)>,
}

struct Job {
    child: std::process::Child,
    status: String,
    job_number: u32,
    process_id: u32,
    command: String,
}

fn parse_redirect(input: &str) -> (String, Redirect) {
    if let Some(pos) = input.find("1>>") {
        let cmd = input[..pos].trim().to_string();
        let file = input[pos + 3..].trim().to_string();
        return (
            cmd,
            Redirect {
                stdout: Some((file, RedirectMode::Append)),
                stderr: None,
            },
        );
    } else if let Some(pos) = input.find("2>>") {
        let cmd = input[..pos].trim().to_string();
        let file = input[pos + 3..].trim().to_string();
        return (
            cmd,
            Redirect {
                stdout: None,
                stderr: Some((file, RedirectMode::Append)),
            },
        );
    } else if let Some(pos) = input.find(">>") {
        let cmd = input[..pos].trim().to_string();
        let file = input[pos + 2..].trim().to_string();
        return (
            cmd,
            Redirect {
                stdout: Some((file, RedirectMode::Append)),
                stderr: None,
            },
        );
    } else if let Some(pos) = input.find("2>") {
        let cmd = input[..pos].trim().to_string();
        let file = input[pos + 2..].trim().to_string();
        return (
            cmd,
            Redirect {
                stdout: None,
                stderr: Some((file, RedirectMode::Overwrite)),
            },
        );
    } else if let Some(pos) = input.find("1>") {
        let cmd = input[..pos].trim().to_string();
        let file = input[pos + 2..].trim().to_string();
        return (
            cmd,
            Redirect {
                stdout: Some((file, RedirectMode::Overwrite)),
                stderr: None,
            },
        );
    } else if let Some(pos) = input.find(">") {
        let cmd = input[..pos].trim().to_string();
        let file = input[pos + 1..].trim().to_string();
        return (
            cmd,
            Redirect {
                stdout: Some((file, RedirectMode::Overwrite)),
                stderr: None,
            },
        );
    }

    (
        input.trim().to_string(),
        Redirect {
            stdout: None,
            stderr: None,
        },
    )
}

fn main() {
    let mut jobs: Vec<Job> = Vec::new();

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        command = command.trim().to_string();

        let (command, redirect) = parse_redirect(&command);
        let command = command.as_str();

        let stdout_file = redirect.stdout.as_ref().map(|(filename, mode)| match mode {
            RedirectMode::Overwrite => File::create(filename).unwrap(),
            RedirectMode::Append => File::options()
                .append(true)
                .create(true)
                .open(filename)
                .unwrap(),
        });

        let stderr_file = redirect.stderr.as_ref().map(|(filename, mode)| match mode {
            RedirectMode::Overwrite => File::create(filename).unwrap(),
            RedirectMode::Append => File::options()
                .append(true)
                .create(true)
                .open(filename)
                .unwrap(),
        });

        if command == "exit" {
            break;
        }

        if command.starts_with("echo ") {
            let parts = parse_args(&command[5..]);
            let output = parts.join(" ");

            match stdout_file {
                Some(mut file) => writeln!(file, "{}", output).unwrap(),
                None => println!("{}", output),
            }
        } else if command.starts_with("type ") {
            let cmd = command.split_whitespace().nth(1).unwrap();

            match cmd {
                "echo" | "exit" | "type" | "pwd" | "cd" | "jobs" => {
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
        } else if command == "jobs" {
            let len = jobs.len();
            for (index, job) in jobs.iter_mut().enumerate() {
                let mut marker;
                if index == len - 1 {
                    marker = '+';
                } else if index == len - 2 {
                    marker = '-';
                } else {
                    marker = ' ';
                }

                if let Ok(Some(_)) = job.child.try_wait() {
                    job.status = "Done".to_string();
                }

                if job.status == "Done" {
                    println!(
                        "[{}]{}  {:<24}{} ",
                        index + 1,
                        marker,
                        job.status,
                        job.command
                    );
                } else if job.status == "Running" {
                    println!(
                        "[{}]{}  {:<24}{} &",
                        index + 1,
                        marker,
                        job.status,
                        job.command
                    );
                }
            }

            jobs.retain(|job| job.status != "Done")
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
            let mut parts = parse_args(&command);
            let background = parts.last().map(|x| x == "&").unwrap_or(false);
            if background {
                parts.pop();
            }
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

            if let Some(_path) = found_path {
                let mut cmd = std::process::Command::new(program); // ✅
                cmd.args(args);

                if let Some(file) = stdout_file {
                    cmd.stdout(file);
                }
                if let Some(file) = stderr_file {
                    cmd.stderr(file);
                }

                if background {
                    let child = cmd.spawn().unwrap();
                    let job_number = jobs.len() as u32 + 1;
                    let pid = child.id();
                    println!("[{}] {}", job_number, pid);
                    jobs.push(Job {
                        child: child,
                        status: "Running".to_string(),
                        job_number,
                        process_id: pid,
                        command: parts.join(" "),
                    });
                } else {
                    let mut child = cmd.spawn().unwrap();
                    child.wait().unwrap();
                }
            } else {
                println!("{}: not found", program);
            }
        }
    }
}
