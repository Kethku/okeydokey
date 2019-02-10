#[macro_use]
extern crate clap;
use clap::App;

use std::fs::File;
use std::env;
use std::path::{PathBuf};
use std::io::{BufReader, BufRead};

struct Profile {
    commands: Vec<(String, String)>,
    path: PathBuf
}

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let profile_opt = find_profile(env::current_dir().unwrap());
    if profile_opt.is_some() {
        let profile = profile_opt.unwrap();

        match matches.value_of("COMMAND") {
            Some(command) => query(
                profile,
                command,
                matches.value_of("prefix"),
                matches.value_of("sufix")),
            None => list(profile)
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

            for line in BufReader::new(file).lines() {
                let command_parts = split_on_colon(line.unwrap())?;
                commands.push(command_parts);
            }

            Some(Profile { commands, path: profile_path })
        },
        Err(_) => None
    }
}

fn split_on_colon(line: String) -> Option<(String, String)> {
    let mut splitter = line.splitn(2, ':');
    let name = splitter.next()?;
    let command = splitter.next()?;
    Some((name.to_string(), command.to_string()))
}


fn list(profile: Profile) {
    let list = profile.commands
        .iter()
        .map(|(name, _)| name)
        .fold(String::new(), |acc, next| {
            acc + " " + next
        });
    println!("{}", list.trim());
}

fn query(profile: Profile, command: &str, prefix: Option<&str>, sufix: Option<&str>) {
    let best_option = profile.commands
        .iter()
        .filter_map(|(possible_command, _)| shared_prefix(possible_command, command))
        .max_by_key(|&(shared_chars, _)| shared_chars);

    match best_option {
        Some((_, actual_command)) => print_decorated_command(profile, actual_command, prefix, sufix),
        None => ()
    }
}

fn shared_prefix(possible_command: &str, command: &str) -> Option<(usize, String)> {
    match possible_command.starts_with(command) {
        true => Some((command.len(), possible_command.to_string())),
        false => None
    }
}

fn print_decorated_command(profile: Profile, command_name: String, prefix: Option<&str>, sufix: Option<&str>) {
    let prefix = fill_in_profile_directory(&profile, prefix);
    let sufix = fill_in_profile_directory(&profile, sufix);
    let (_, command) = profile.commands
        .into_iter()
        .find(|(name, _)| *name == command_name)
        .unwrap();

    println!("{}", vec![prefix, command.to_string(), sufix].concat())
}

fn fill_in_profile_directory(profile: &Profile, pattern: Option<&str>) -> String {
    let profile_directory = profile.path.parent().unwrap().to_str().unwrap();
    pattern.unwrap_or_default().replace("{}", profile_directory)
}
