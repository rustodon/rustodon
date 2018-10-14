extern crate posticle;

use posticle::Posticle;
use std::io;
use std::io::prelude::*;
use std::io::Write;

fn main() {
    let posticle = Posticle::new();
    let stdin = io::stdin();

    println!("Posticle token debug tool.");
    print!("> ");
    io::stdout().flush().unwrap();

    for line in stdin.lock().lines() {
        let text = line.unwrap();
        let entities = posticle.parse(&text);

        for entity in entities {
            println!("{:#?}", entity);
        }

        print!("\n> ");
        io::stdout().flush().unwrap();
    }
}
