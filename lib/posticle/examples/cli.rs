extern crate posticle;

use std::io;
use std::io::prelude::*;


fn main() {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let text = line.unwrap();
        let entities = posticle::entities(&text);
        for entity in entities {
            println!("entity: {:#?}", entity);
            println!("raw text: {}", entity.substr(&text));
        }
    }
}
