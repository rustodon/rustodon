extern crate posticle;

use posticle::tokens::*;
use posticle::Reader;

struct Blacklist(Vec<&'static str>);

impl Blacklist {
    fn test(&self, reader: Reader) -> bool {
        for token in reader {
            match token {
                Token::Text(text) => {
                    if self.test_text(&text.0) {
                        return true;
                    }
                },
                _ => {},
            }
        }

        false
    }

    fn test_text(&self, text: &str) -> bool {
        for word in &self.0 {
            if text.contains(word) {
                return true;
            }
        }

        false
    }
}

fn main() {
    let blacklist = Blacklist(vec!["pants"]);
    let reader = Reader::from("Developer is not wearing any pants!?");

    println!("Is blacklisted: {:?}", blacklist.test(reader));
}
