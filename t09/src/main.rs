use anyhow::{anyhow, Result};
use itertools::Itertools;
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    iter::Peekable,
    path::PathBuf,
    process::Stdio,
    str::{Chars, FromStr},
};

fn main() {
    let mut shell_envs = HashMap::new();

    loop {
        // { ENVIRONMENT_VARIABLE=...,ENVIRONMENT_VARIABLE=... } { command arg ... arg } { < filepath OR > filepath OR >> filepath} | ...
        let input = match get_input() {
            Ok(input) => input,
            Err(err) => {
                eprintln!("failed to read command: {err}");
                continue;
            }
        };
        if input.is_empty() {
            continue;
        }

        if let Err(err) = process_commands(&input, &mut shell_envs) {
            eprintln!("{err}");
        };
    }
}

fn process_commands(input: &str, shell_envs: &mut HashMap<String, String>) -> Result<()> {
    let input = input.trim();
    // split commands by pipelines
    let commands = input.split('|').map(str::trim).collect::<Vec<_>>();
    let commands_count = commands.len();
    let mut previous_output = None;

    for (command_number, command) in commands.iter().enumerate() {
        let mut command = command.chars().peekable();

        let mut envs = get_envs(&mut command)?;
        let command = command.collect::<String>();
        // now we have separeted envs and command

        // if input only contains envs; example: "ENV_VAR_1=VALUE,ENV_VAR_2=VALUE"
        if command.is_empty() {
            shell_envs.extend(envs);
            return Ok(());
        }

        envs.extend(shell_envs.clone());

        // get information about redirects
        // if redirect in command, then it splits command into command with arguments and redirection information
        match check_redirects(&command) {
            Ok(None) => {
                execute_command_without_redirects(
                    &command,
                    command_number,
                    commands_count,
                    &mut previous_output,
                    &envs,
                )?;
            }
            Ok(Some((command, redirect))) => {
                execute_command_with_redirects(
                    command,
                    redirect,
                    command_number,
                    commands_count,
                    &mut previous_output,
                    &envs,
                )?;
            }
            Err(err) => {
                return Err(anyhow!("{err}"));
            }
        };
    }

    Ok(())
}

#[allow(clippy::too_many_lines)]
fn execute_command_with_redirects(
    command: &str,
    mut redirect: Redirect,
    command_number: usize,
    commands_count: usize,
    previous_output: &mut Option<String>,
    envs: &HashMap<String, String>,
) -> Result<(), anyhow::Error> {
    let parts = command.split_whitespace().collect::<Vec<_>>();
    match parts.as_slice() {
        ["cd", args @ ..] => {
            if let Redirect::Input { contents } = redirect {
                let first = match args.first() {
                    Some(&first) => first,
                    None => contents.as_str(),
                };

                if let Err(err) = std::env::set_current_dir(first) {
                    return Err(anyhow!("cd: {err}: {first}"));
                }
            } else {
                let path = match args.first() {
                    Some(&path) => PathBuf::from_str(path),
                    None => PathBuf::from_str(""),
                }
                .expect("Err in PathBuf::from_str is infallible");

                if let Err(err) = std::env::set_current_dir(&path) {
                    return Err(anyhow!(
                        "cd: {err}: {}",
                        path.to_str().expect("was previously str already")
                    ));
                }
            }
        }
        ["ls", args @ ..] => match redirect {
            Redirect::Input { contents: _ } => {
                let path = match args.first() {
                    Some(&path) => std::path::PathBuf::from_str(path)
                        .expect("Err in PathBuf::from_str is infallible"),
                    None => match std::env::current_dir() {
                        Ok(current_dir) => current_dir,
                        Err(err) => {
                            return Err(anyhow!("ls: {err}"));
                        }
                    },
                };

                let entries = match std::fs::read_dir(path) {
                    Ok(entries) => entries,
                    Err(err) => {
                        return Err(anyhow!("ls: {err}"));
                    }
                };

                let entries_list = entries.process_results(|iter| {
                    iter.map(|entry| entry.file_name().to_string_lossy().into_owned())
                        .join("\n")
                })?;

                if command_number == commands_count - 1 {
                    println!("{entries_list}");
                } else {
                    *previous_output = Some(entries_list);
                }
            }
            Redirect::Output { mut output_file } => {
                let path = match args.first() {
                    Some(&path) => std::path::PathBuf::from_str(path)
                        .expect("Err in PathBuf::from_str is infallible"),
                    None => match std::env::current_dir() {
                        Ok(current_dir) => current_dir,
                        Err(err) => {
                            return Err(anyhow!("ls: {err}"));
                        }
                    },
                };

                let entries = match std::fs::read_dir(path) {
                    Ok(entries) => entries,
                    Err(err) => {
                        return Err(anyhow!("ls: {err}"));
                    }
                };

                let entries_list = entries.process_results(|iter| {
                    iter.map(|entry| entry.file_name().to_string_lossy().into_owned())
                        .join("\n")
                })?;

                if let Err(err) = output_file.write_all(entries_list.as_bytes()) {
                    return Err(anyhow!("ls: {err}"));
                };
            }
            Redirect::Append { mut output_file } => {
                let path = match args.first() {
                    Some(&path) => std::path::PathBuf::from_str(path)
                        .expect("Err in PathBuf::from_str is infallible"),
                    None => match std::env::current_dir() {
                        Ok(current_dir) => current_dir,
                        Err(err) => {
                            return Err(anyhow!("ls: {err}"));
                        }
                    },
                };

                let entries = match std::fs::read_dir(path) {
                    Ok(entries) => entries,
                    Err(err) => {
                        return Err(anyhow!("ls: {err}"));
                    }
                };

                let entries_list = entries.process_results(|iter| {
                    iter.map(|entry| entry.file_name().to_string_lossy().into_owned())
                        .join("\n")
                })?;

                if let Err(err) = output_file.write_all(entries_list.as_bytes()) {
                    return Err(anyhow!("ls: {err}"));
                };
            }
        },
        ["pwd", ..] => match redirect {
            Redirect::Input { contents: _ } => {
                let current_path = match std::env::current_dir() {
                    Ok(path) => format!("{}", path.display()),
                    Err(err) => {
                        return Err(anyhow!("pwd: {err}"));
                    }
                };

                if command_number == commands_count - 1 {
                    println!("{current_path}");
                } else {
                    *previous_output = Some(current_path);
                }
            }
            Redirect::Output { mut output_file } | Redirect::Append { mut output_file } => {
                let current_path = match std::env::current_dir() {
                    Ok(path) => format!("{}", path.display()),
                    Err(err) => {
                        return Err(anyhow!("pwd: {err}"));
                    }
                };

                output_file.write_all(current_path.as_bytes())?;
            }
        },
        ["echo", args @ ..] => match redirect {
            Redirect::Input { contents: _ } => {
                let args = args.join(" ");

                if command_number == commands_count - 1 {
                    println!("{args}");
                } else {
                    *previous_output = Some(args);
                }
            }
            Redirect::Output { mut output_file } | Redirect::Append { mut output_file } => {
                let args = args.join(" ");

                output_file.write_all(args.as_bytes())?;
            }
        },
        [command, args @ ..] => {
            let mut cmd = std::process::Command::new(command);

            cmd.stdin(Stdio::piped());
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());

            cmd.envs(envs);
            cmd.args(args);

            // spawn process
            let mut child = match cmd.spawn() {
                Ok(child) => child,
                Err(err) => {
                    return Err(anyhow!("failed to spawn command '{command}': {err}"));
                }
            };

            {
                let Some(mut stdin) = child.stdin.take() else {
                    return Err(anyhow!("failed to take child's stdin"));
                };

                if let Redirect::Input { ref mut contents } = redirect {
                    stdin.write_all(contents.as_bytes())?;
                } else if let Some(previous_output) = previous_output.take() {
                    // write previous output to child stdin
                    stdin.write_all(previous_output.as_bytes())?;
                }
            }

            // wait and obtain stdout and stderr
            let output = child.wait_with_output()?;
            let output = String::from_utf8(output.stdout)?;

            if let Redirect::Output { mut output_file } | Redirect::Append { mut output_file } =
                redirect
            {
                output_file.write_all(output.as_bytes())?;
            } else if command_number == commands_count - 1 {
                println!("{output}");
            } else {
                *previous_output = Some(output);
            }
        }
        [] => {}
    };
    Ok(())
}

fn execute_command_without_redirects(
    command: &str,
    command_number: usize,
    commands_count: usize,
    previous_output: &mut Option<String>,
    envs: &HashMap<String, String>,
) -> Result<(), anyhow::Error> {
    let parts = command.split_whitespace().collect::<Vec<_>>();
    match parts.as_slice() {
        ["cd", args @ ..] => {
            let first = args.first().unwrap_or(&"");

            if let Err(err) = std::env::set_current_dir(first) {
                return Err(anyhow!("cd: {err}: {first}"));
            }
        }
        ["ls", args @ ..] => {
            let path = match args.first() {
                Some(&path) => std::path::PathBuf::from_str(path)
                    .expect("Err in PathBuf::from_str is infallible"),
                None => match std::env::current_dir() {
                    Ok(current_dir) => current_dir,
                    Err(err) => {
                        return Err(anyhow!("ls: {err}"));
                    }
                },
            };

            let entries = match std::fs::read_dir(path) {
                Ok(entries) => entries,
                Err(err) => {
                    return Err(anyhow!("ls: {err}"));
                }
            };

            let entries_list = entries.process_results(|iter| {
                iter.map(|entry| entry.file_name().to_string_lossy().into_owned())
                    .join("\n")
            })?;

            if command_number == commands_count - 1 {
                println!("{entries_list}");
            } else {
                *previous_output = Some(entries_list);
            }
        }
        ["pwd", ..] => {
            let current_path = match std::env::current_dir() {
                Ok(path) => format!("{}", path.display()),
                Err(err) => {
                    return Err(anyhow!("pwd: {err}"));
                }
            };

            if command_number == commands_count - 1 {
                println!("{current_path}");
            } else {
                *previous_output = Some(current_path);
            }
        }
        ["echo", args @ ..] => {
            let args = args.join(" ");

            if command_number == commands_count - 1 {
                println!("{args}");
            } else {
                *previous_output = Some(args);
            }
        }
        [command, args @ ..] => {
            let mut cmd = std::process::Command::new(command);

            cmd.stdin(Stdio::piped());
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());

            cmd.envs(envs);
            cmd.args(args);

            // spawn process
            let mut child = match cmd.spawn() {
                Ok(child) => child,
                Err(err) => return Err(anyhow!("failed to spawn command '{command}': {err}")),
            };

            // write previous output to child stdin
            if let Some(previous_output) = previous_output.take() {
                let Some(mut stdin) = child.stdin.take() else {
                    return Err(anyhow!("failed to take child's stdin"));
                };

                stdin.write_all(previous_output.as_bytes())?;
            }

            // wait and obtain stdout and stderr
            let output = child.wait_with_output()?;

            let output = String::from_utf8(output.stdout)?;

            if command_number == commands_count - 1 {
                println!("{output}");
            } else {
                *previous_output = Some(output);
            }
        }
        [] => unreachable!(),
    };
    Ok(())
}

#[derive(Debug)]
enum State {
    Start,
    Key(String),
    KeyValue(String, String),
}

fn get_envs(command: &mut Peekable<Chars<'_>>) -> Result<HashMap<String, String>> {
    let mut envs = HashMap::new();
    let mut state = State::Start;

    while let Some(&peeked_char) = command.peek() {
        state = match (state, peeked_char) {
            (State::Start, char) => match char {
                lowercase if lowercase.is_lowercase() => {
                    return Ok(envs);
                }
                alphabetic if alphabetic.is_ascii_alphabetic() => State::Key(char.to_string()),
                char => return Err(anyhow!("environment variable cannot start with: '{char}'")),
            },
            (State::Key(mut key), key_char) => {
                match key_char {
                    lowercase if lowercase.is_lowercase() => {
                        return Err(anyhow!("environment variable cannot contain lowercase character: '{lowercase}'"));
                    }
                    '=' => State::KeyValue(key, String::new()),
                    key_char => {
                        key.push(key_char);
                        State::Key(key)
                    }
                }
            }
            (State::KeyValue(key, mut value), value_char) => match value_char {
                ',' => {
                    envs.insert(key, value);
                    State::Start
                }
                ' ' => {
                    envs.insert(key, value);
                    command
                        .next()
                        .expect("next char exists due to peek() is Some(char)");
                    return Ok(envs);
                }
                char => {
                    value.push(char);
                    State::KeyValue(key, value)
                }
            },
        };
        command
            .next()
            .expect("next char exists due to peek() is Some(char)");
    }

    match state {
        State::Start => Ok(envs),
        State::Key(key) => Err(anyhow!("missing value for key: '{key}'")),
        State::KeyValue(key, value) => {
            envs.insert(key, value);
            Ok(envs)
        }
    }
}

#[derive(Debug)]
enum Redirect {
    // <
    Input { contents: String },
    // >
    Output { output_file: File },
    // >>
    Append { output_file: File },
}

fn check_redirects(command: &str) -> Result<Option<(&str, Redirect)>> {
    match (
        command.split_once('<'),
        command.split_once('>'),
        command.split_once("<<"),
    ) {
        (None, None, None) => Ok(None),
        (Some((command, input_filepath)), None, None) if !input_filepath.contains('<') => {
            let mut input_file = std::fs::File::open(input_filepath)?;
            let mut contents = String::new();
            input_file.read_to_string(&mut contents)?;
            Ok(Some((command, Redirect::Input { contents })))
        }
        (None, Some((command, output_filepath)), None) if !output_filepath.contains('>') => {
            let output_file = std::fs::File::create(output_filepath)?;
            Ok(Some((command, Redirect::Output { output_file })))
        }
        (None, None, Some((command, output_filepath))) if !output_filepath.contains(">>") => {
            let output_file = std::fs::File::options()
                .append(true)
                .create(true)
                .open(output_filepath)?;

            Ok(Some((command, Redirect::Append { output_file })))
        }
        _ => Err(anyhow!(
            "cannot have multiple redirects in command: '{command}'"
        )),
    }
}

fn get_input() -> Result<String, std::io::Error> {
    let current_directory = std::env::current_dir().unwrap_or_default();
    print!("{}> ", current_directory.display());
    std::io::stdout().flush().expect("all bytes written");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(input)
}
