//! lovingly stolen from https://github.com/rust-lang-nursery/rustup.rs/blob/master/build.rs

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::{self, Command};

struct Ignore;

impl<E> From<E> for Ignore
where
    E: Error,
{
    fn from(_: E) -> Ignore {
        Ignore
    }
}

fn main() {
    if let Err(_) = Command::new("sass").status() {
        eprintln!("build error: sass compiler not installed. please run `gem install sass`.");
        process::exit(1);
    }

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    File::create(out_dir.join("commit-info.txt"))
        .unwrap()
        .write_all(commit_info().as_bytes())
        .unwrap();

    Command::new("sass")
        .args(&["style/main.scss", "static/style.css"])
        .status()
        .unwrap();
}

// Try to get hash and date of the last commit on a best effort basis. If anything goes wrong
// (git not installed or if this is not a git repository) just return an empty string.
fn commit_info() -> String {
    match commit_hash() {
        Ok(hash) => hash.trim_right().to_owned(),
        _ => String::new(),
    }
}

fn commit_hash() -> Result<String, Ignore> {
    Ok(String::from_utf8(
        Command::new("git")
            .args(&["rev-parse", "--short=9", "HEAD"])
            .output()?
            .stdout,
    )?)
}
