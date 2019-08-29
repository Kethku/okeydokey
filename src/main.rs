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

    let profiles = find_profiles(env::current_dir().unwrap());

    match matches.value_of("COMMAND") {
        Some(command_prefix) => {
            let prefix = matches.value_of("prefix");
            let suffix = matches.value_of("suffix");
            let args = matches.values_of("args").map_or(Vec::new(), |args| args.collect());
            run(profiles, command_prefix, prefix, suffix, args);
        }
        None => list(profiles)
    };
}

fn run(profiles: Vec<Profile>, command_prefix: &str, prefix: Option<&str>, suffix: Option<&str>, args: Vec<&str>) -> Option<()> {
    let (command, profile_path) = query(profiles, command_prefix)?;
    print_decorated_command(
        command, 
        profile_path.clone(), 
        prefix,
        suffix,
        args);
    Some(())
}

fn find_profiles(initial_path: PathBuf) -> Vec<Profile> {
    let mut profile_paths = Vec::new();

    let mut current_path = initial_path;
    loop {
        let possible_profile = current_path.join(".ok");
        if possible_profile.exists() {
            profile_paths.push(possible_profile);
        }

        let parent_path = current_path.parent();
        if parent_path.is_some() {
            current_path = parent_path.unwrap().to_path_buf();
        } else {
            break;
        }
    }

    return profile_paths.into_iter().map(|path| read_profile(path)).filter_map(|x| x).collect();
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


fn list(profiles: Vec<Profile>) {
    let list = profiles.iter()
        .flat_map(|profile| profile.commands.iter())
        .map(|(name, _)| name)
        .fold(String::new(), |acc, next| {
            acc + " " + next
        });
    println!("{}", list.trim());
}

fn query(profiles: Vec<Profile>, command_prefix: &str) -> Option<(String, PathBuf)> {
    profiles.into_iter()
        .flat_map(|profile| {
            let path = profile.path.clone();
            profile.commands.into_iter().map(move |command| (command.clone(), path.clone()))
        })
        .filter_map(|((possible_command_name, command), profile_path)| shared_prefix(&possible_command_name, command_prefix).map(|result| (result, command, profile_path)))
        .max_by_key(|&((shared_chars, _), _, _)| shared_chars)
        .map(|(_, command, path)| (command, path))
}

fn shared_prefix(possible_command: &str, command: &str) -> Option<(usize, String)> {
    match possible_command.starts_with(command) {
        true => Some((command.len(), possible_command.to_string())),
        false => None
    }
}

fn print_decorated_command(command: String, profile_path: PathBuf, prefix: Option<&str>, suffix: Option<&str>, args: Vec<&str>) {
    let prefix = fill_in_profile_directory(&profile_path, prefix);
    let suffix = fill_in_profile_directory(&profile_path, suffix);
    println!("{}", vec![prefix, fill_in_arguments(command.to_string(), args), suffix].concat())
}

fn fill_in_profile_directory(profile_path: &PathBuf, pattern: Option<&str>) -> String {
    let profile_directory = profile_path.parent().unwrap().to_str().unwrap();
    pattern.unwrap_or_default().replace("{}", profile_directory)
}

fn hole(n: usize) -> String {
    format!("{{{}}}", n)
}

fn count_holes(command: &String) -> usize {
    fn rec(command: &String, n: usize) -> usize {
        match command.contains(&hole(n)[..]) {
            true => rec(command, n + 1),
            false => n
        }
    }

    rec(command, 0)
}

fn fill_in_arguments(perferated_command: String, args: Vec<&str>) -> String {
    let number_of_holes = count_holes(&perferated_command);
    let mut args_iterator = args.iter();
    let mut command = perferated_command;

    for hole_number in 0..number_of_holes {
        let hole_string = hole(hole_number);
        command = match args_iterator.next() {
            Some(arg) => command.replace(&hole_string[..], arg),
            None => command.replace(&hole_string[..], "")
        };
    }

    for arg in args_iterator {
        command = command + " " + arg;
    }

    command
}
