use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use clap::Parser;
use indoc::indoc;

struct Profile {
    commands: Vec<(String, String)>,
    internal_commands: Vec<(String, String)>,
    path: PathBuf,
}

impl Profile {
    fn all_commands(&self) -> Vec<(String, String)> {
        let mut commands = self.commands.clone();
        commands.extend(self.internal_commands.clone());
        commands
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(help = "The command in the profile to run")]
    command: Option<String>,
    #[clap(
        short,
        long,
        help = "Prepends argument to the returned command replacing {} with the full path to the found .ok file."
    )]
    prefix: Option<String>,
    #[clap(
        short,
        long,
        help = "Appends argument to the returned command replacing {} with the full path to the found .ok file."
    )]
    suffix: Option<String>,
    #[clap(
        short,
        long,
        num_args = 0..,
        allow_hyphen_values = true,
        help = "Fills {n} in the matched command with the nth arguments in this list. If less than n arguments provided, empty string is substituted instead. If more than the total holes in the command are provided, then the arguments are appended to the command separated by spaces."
    )]
    args: Vec<String>,

    #[clap(
        short,
        long,
        help = "Whether to instead of running a command, print out the hook for the given platform shell."
    )]
    hook: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    if let Some(shell) = cli.hook {
        let hook = match shell.as_ref() {
            "pwsh" => Some(indoc! {r#"
                function ok
                {
                    if ($args.Count -eq 0) {
                        okeydokey | Write-Host -ForegroundColor 'Blue'
                    } else {
                        if ($args.Count -gt 1) {
                            $script = okeydokey $args[0] -p "pushd {};" -s "; popd" -a ($args | select -skip 1)
                        } else {
                            $script = okeydokey $args[0] -p "pushd {};" -s "; popd"
                        }

                        if ($script -ne $null) {
                            iex $script
                        }
                    }
                }
            "#}),
            _ => None,
        };

        if let Some(hook) = hook {
            println!("{}", hook);
        } else {
            eprintln!("Invalid hook. Try pwsh");
        }
        return;
    }

    let profile_opt = find_profile(env::current_dir().unwrap());
    if profile_opt.is_some() {
        let profile = profile_opt.unwrap();

        if let Some(command) = cli.command {
            query(profile, command, cli.prefix, cli.suffix, cli.args);
        } else {
            list(profile);
        }
    }
}

fn find_profile(current_path: PathBuf) -> Option<Profile> {
    let possible_profile = current_path.join(".ok");
    if possible_profile.exists() {
        Some(read_profile(possible_profile)?)
    } else {
        Some(find_profile(current_path.parent()?.to_path_buf())?)
    }
}

fn read_profile(profile_path: PathBuf) -> Option<Profile> {
    match File::open(profile_path.clone()) {
        Ok(ref mut file) => {
            let mut commands = Vec::new();
            let mut internal_commands = Vec::new();

            for line in BufReader::new(file).lines() {
                let (name, command) = split_on_colon(line.unwrap())?;
                if name.starts_with("_") {
                    internal_commands.push((name, command));
                } else {
                    commands.push((name, command));
                }
            }

            Some(Profile {
                internal_commands,
                commands,
                path: profile_path,
            })
        }
        Err(_) => None,
    }
}

fn split_on_colon(line: String) -> Option<(String, String)> {
    let mut splitter = line.splitn(2, ':');
    let name = splitter.next()?;
    let command = splitter.next()?;
    Some((name.to_string(), command.to_string()))
}

fn list(profile: Profile) {
    let list = profile
        .commands
        .iter()
        .map(|(name, _)| name)
        .fold(String::new(), |acc, next| acc + " " + next);
    println!("{}", list.trim());
}

fn query(
    profile: Profile,
    command: String,
    prefix: Option<String>,
    suffix: Option<String>,
    args: Vec<String>,
) -> Option<()> {
    let commands_with_valid_prefix_count = profile
        .all_commands()
        .iter()
        .filter_map(|(possible_command, _)| shared_prefix(possible_command, &command))
        .collect::<Vec<_>>();

    let most_shared_chars = commands_with_valid_prefix_count
        .iter()
        .cloned()
        .map(|(shared_chars, _)| shared_chars)
        .max()?;

    let best_command = commands_with_valid_prefix_count
        .into_iter()
        .filter(|(shared_chars, _)| *shared_chars == most_shared_chars)
        .map(|(_, command)| command)
        .min_by_key(|command| command.len())?;

    print_decorated_command(profile, best_command, prefix, suffix, args);
    Some(())
}

fn shared_prefix(possible_command: &str, command: &str) -> Option<(usize, String)> {
    match possible_command.starts_with(command) {
        true => Some((command.len(), possible_command.to_string())),
        false => None,
    }
}

fn print_decorated_command(
    profile: Profile,
    command_name: String,
    prefix: Option<String>,
    suffix: Option<String>,
    args: Vec<String>,
) {
    let prefix = fill_in_profile_directory(&profile, prefix);
    let suffix = fill_in_profile_directory(&profile, suffix);
    let (_, command) = profile
        .all_commands()
        .into_iter()
        .find(|(name, _)| *name == command_name)
        .unwrap();

    println!(
        "{}",
        vec![prefix, fill_in_arguments(command.to_string(), args), suffix].concat()
    )
}

fn fill_in_profile_directory(profile: &Profile, pattern: Option<String>) -> String {
    let profile_directory = profile.path.parent().unwrap().to_str().unwrap();
    pattern.unwrap_or_default().replace("{}", profile_directory)
}

fn hole(n: usize) -> String {
    format!("{{{}}}", n)
}

fn count_holes(command: &String) -> usize {
    fn rec(command: &String, n: usize) -> usize {
        match command.contains(&hole(n)[..]) {
            true => rec(command, n + 1),
            false => n,
        }
    }

    rec(command, 0)
}

fn fill_in_arguments(perferated_command: String, args: Vec<String>) -> String {
    let number_of_holes = count_holes(&perferated_command);
    let mut args_iterator = args.iter();
    let mut command = perferated_command;

    for hole_number in 0..number_of_holes {
        let hole_string = hole(hole_number);
        command = match args_iterator.next() {
            Some(arg) => command.replace(&hole_string[..], arg),
            None => command.replace(&hole_string[..], ""),
        };
    }

    for arg in args_iterator {
        command = command + " " + arg;
    }

    command
}
