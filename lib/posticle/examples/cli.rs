extern crate posticle;

use posticle::Posticle;
use std::io;
use std::io::prelude::*;

fn main() {
    let posticle = Posticle::new();
    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        let text = line.unwrap();
        let entities = posticle.parse(&text);

        for entity in entities {
            println!("entity: {:#?}", entity);
        }
    }
}
