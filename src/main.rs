use std::env;
use std::fs::File;
use std::io::{Read, copy, stdin, stdout};
use std::process::Command;

extern crate tempfile;

use tempfile::NamedTempFile;


fn lookup_program() -> String {
    match env::var_os("VISUAL") {
        Some(val) => val,
        None => env::var_os("EDITOR").expect("Set $VISUAL or $EDITOR to use edfp")
    }.into_string().unwrap()
}

fn edfp<I: Read>(mut input: I) -> File {
    let mut tmp: NamedTempFile = NamedTempFile::new().expect("failed to create temporary file");
    copy(&mut input, &mut tmp).unwrap();

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

    tmp.reopen().ok().expect("failed to re-open")
}

fn main() {
    let input = stdin();
    let mut output = stdout();
    copy(&mut edfp(input), &mut output).unwrap();
}
