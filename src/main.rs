use std::env;
use std::fmt;
use std::fs::{OpenOptions, File, rename, remove_file};
use std::io::{BufRead, Write};
use std::io;
use std::path::PathBuf;
use std::process::Command;

extern crate tempfile;

use tempfile::NamedTempFile;

#[derive(Debug)]
enum Operation {
    NoOp,
    Remove,
    Rename { new_path: PathBuf }
}

#[derive(Debug)]
struct Change {
    path: PathBuf,
    operation: Operation
}

trait Operate {
    fn operate(&self) -> io::Result<()>;
}

impl Operate for Change {
    fn operate(&self) -> io::Result<()> {
        match self.operation {
            Operation::NoOp => Ok(()),
            Operation::Remove => remove_file(&self.path),
            Operation::Rename { ref new_path } => rename(&self.path, new_path)
        }
    }
}

impl fmt::Display for Change {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.operation {
            Operation::NoOp => write!(f, ""),
            Operation::Remove => write!(f, "{}", &self.path.to_str().unwrap()),
            Operation::Rename { ref new_path } => write!(f, "{} → {}", &self.path.to_str().unwrap(), new_path.to_str().unwrap())
        }
    }
}

pub fn get_tty() -> io::Result<File> {
    OpenOptions::new().read(true).write(true).open("/dev/tty")
}

fn lookup_program() -> String {
    match env::var_os("VISUAL") {
        Some(val) => val,
        None => env::var_os("EDITOR").expect("Set $VISUAL or $EDITOR to use edfp")
    }.into_string().unwrap()
}

fn parse_lines(lines: (String, String)) -> Change {
    let (given, new) = lines;
    let path = PathBuf::from(&given);
    let empty_string = String::new();
    if new == given {
        Change { operation: Operation::NoOp, path }
    } else if new == String::new() {
        Change { operation: Operation::Remove, path }
    } else {
        Change {
            operation: Operation::Rename {
                new_path: PathBuf::from(new)
            },
            path
        }
    }
}

fn describe_modifying_changes<O: io::Write>(header: &str, changes: &Vec<&Change>, mut output: O) -> O {
    if !changes.is_empty() {
        writeln!(output, "{}", header);
        for change in changes {
            writeln!(output, "\t{}", change);
        }
        writeln!(output);
    }
    output
}

fn describe_changes<O: io::Write>(changes: &Vec<Change>, mut output: O) -> bool {
    let removals: Vec<&Change> = changes.iter().filter(|c| match c.operation {
        Operation::Remove => true,
        _ => false
    }).collect();
    let renamings: Vec<&Change> = changes.iter().filter(|c| match c.operation {
        Operation::Rename { ref new_path } => true,
        _ => false
    }).collect();

    if renamings.is_empty() && removals.is_empty() {
        writeln!(output, "No changes");
        false
    } else {
        let output = describe_modifying_changes("DELETE the following files:\n", &removals, output);
        describe_modifying_changes("RENAME the following files:\n", &renamings, output);
        true
    }
}


fn user_approves_changes() -> bool {
    let mut input = String::new();
    let tty: File = get_tty().ok().expect("failed to open tty");
    write!(&tty, "Would you like to make the changes described above (y/n)? ");
    io::BufReader::new(tty).read_line(&mut input).ok().expect("failed to read user input");
    input.to_lowercase().starts_with("y")
}

fn edfp<I: io::Read, O: io::Write>(mut input: I, mut output: O) {
    let mut buf = String::new();
    input.read_to_string(&mut buf).expect("Failed to read input");

    let mut tmp: NamedTempFile = NamedTempFile::new().expect("failed to create temporary file");
    tmp.write_all(&buf.as_bytes()).expect("Could not write to tempfile");

    let editor_program = lookup_program();
    let tmp_path_str = tmp.path().to_str().expect("failed to parse tmp path");
    let editor = Command::new(editor_program.as_str()).arg(tmp_path_str).spawn();

    let success = editor
        .ok()
        .expect(format!("failed to launch {}", editor_program).as_str())
        .wait()
        .expect(format!("failed to wait for {}", editor_program).as_str())
        .success();

    if !success {
        panic!(format!("{} exited non-zero", editor_program));
    }

    let commands_file = tmp.reopen().ok().expect("failed to re-open");
    let commands = io::BufReader::new(commands_file).lines()
        .map(|line| line.ok().expect("failed to read line from commands file"));
    let changes: Vec<Change> = buf.lines().map(String::from).zip(commands).map(parse_lines).collect();

    let any_changes: bool = describe_changes(&changes, output);

    if any_changes && user_approves_changes() {
        for change in changes {
            change.operate().ok().expect("failed to change")
        }
    }
}

fn main() {
    let input = io::stdin();
    let mut output = io::stdout();
    edfp(input, output)
}
