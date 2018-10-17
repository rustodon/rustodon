extern crate posticle;

use posticle::Reader;
use std::io;
use std::io::prelude::*;
use std::io::Write;

fn main() {
    let stdin = io::stdin();

    println!("Posticle token debug tool.");
    print!("> ");
    io::stdout().flush().unwrap();

    for line in stdin.lock().lines() {
        let text = line.unwrap();

        for token in Reader::from(text) {
            println!("{:#?}", token);
        }

        print!("\n> ");
        io::stdout().flush().unwrap();
    }
}
