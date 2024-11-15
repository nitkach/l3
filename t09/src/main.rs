use std::{
    io::{Read, Write},
    ops::ControlFlow,
    process::Stdio,
};

fn main() {
    loop {
        let input = get_input();
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        // input: { command } { arg } ... { arg } | { command } { arg } ... { arg } | ...
        // Split input into commands by pipe
        let commands = input.split('|').map(str::trim).collect::<Vec<_>>();

        let mut previous_output = None;

        let commands_count = commands.len();

        for (command_number, command) in commands.iter().enumerate() {
            let parts = command.split_whitespace().collect::<Vec<_>>();
            // let cmd = parts.next().expect("at least one part exist");
            // let args = parts.collect::<Vec<_>>();

            // Handle built-in commands
            match parts.as_slice() {
                ["cd", args @ ..] => {
                    let Some(first) = args.first() else {
                        eprintln!("cd: missing argument");
                        continue;
                    };

                    if let Err(err) = std::env::set_current_dir(first) {
                        eprintln!("cd: {first}: {err}");
                    }
                    // if let Some(dir) = args.get(0) {
                    //     if let Err(err) = std::env::set_current_dir(dir) {
                    //         eprintln!("cd: {}", err);
                    //     }
                    // }
                }
                ["pwd", args @ ..] => match std::env::current_dir() {
                    Ok(path) => {
                        if dbg!((command_number == commands_count - 1)) {
                            println!("{}", path.display());
                        } else {
                            previous_output = Some(format!("{}", path.display()));
                        }
                    }
                    Err(err) => eprintln!("pwd: {err}"),
                },
                ["echo", args @ ..] => {
                    if let Some(output_file) =
                        args.iter().position(|&arg| arg == ">" || arg == ">>")
                    {
                        let filename = args[output_file + 1];
                        let mut file = match args[output_file] {
                            ">" => std::fs::File::create(filename).unwrap(),
                            ">>" => std::fs::File::options()
                                .append(true)
                                .create(true)
                                .open(filename)
                                .unwrap(),
                            _ => unreachable!(),
                        };
                        file.write_all(args[0..output_file].join(" ").as_bytes());
                    }
                    let args = args.join(" ");
                    if command_number == commands_count - 1 {
                        println!("{args}");
                    } else {
                        previous_output = Some(args);
                    }
                }
                ["exit", _] => break,
                [command, args @ ..] => {
                    // Create command
                    let mut cmd = std::process::Command::new(command);
                    cmd.args(args);
                    cmd.stdin(Stdio::piped());

                    if command_number < commands_count - 1 {
                        cmd.stdout(std::process::Stdio::piped());
                    } else {
                        // Check for output redirection
                        if let Some(output_file) =
                            args.iter().position(|&arg| arg == ">" || arg == ">>")
                        {
                            let filename = args[output_file + 1];
                            let file = match args[output_file] {
                                ">" => std::fs::File::create(filename).unwrap(),
                                ">>" => std::fs::File::options()
                                    .append(true)
                                    .create(true)
                                    .open(filename)
                                    .unwrap(),
                                _ => unreachable!(),
                            };
                            cmd.stdout(file);
                        }
                    }

                    // Execute command
                    match cmd.spawn() {
                        Ok(mut child) => {
                            // Set up pipes for input/output redirection
                            if let Some(input_file) = args.iter().position(|&arg| arg == "<") {
                                let filename = args[input_file];
                                let file = std::fs::read_to_string(filename).unwrap();
                                let mut stdin = child.stdin.take().unwrap();
                                stdin.write_all(file.as_bytes()).unwrap();
                            } else if let Some(output) = previous_output.take() {
                                let mut stdin = child.stdin.take().unwrap();
                                stdin.write_all(output.as_bytes()).unwrap();
                            }

                            if command_number < commands.len() - 1 {
                                let mut buf = String::new();
                                child
                                    .stdout
                                    .take()
                                    .unwrap()
                                    .read_to_string(&mut buf)
                                    .unwrap();
                                previous_output = Some(buf);
                            }
                            child.wait().unwrap();
                        }
                        Err(err) => eprintln!("{command}: {err}"),
                    }
                }
                _ => unreachable!(),
            }
        }
        // match () {
        //     () => {}
        // }

        // Result<!, !>

        // NonZero<bool>

        // None.unwrap()

        // Arc<Mutex<u8>>
    }
}

fn get_input() -> String {
    let current_directory = std::env::current_dir().unwrap_or_default();
    print!("{}> ", current_directory.display());
    std::io::stdout().flush().expect("all bytes written");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("valid UTF-8");
    input
}
