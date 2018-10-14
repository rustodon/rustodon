extern crate ammonia;
#[macro_use]
extern crate maplit;
extern crate posticle;

use ammonia::{Builder, Url};
use posticle::tokens::*;
use posticle::{Posticle, PosticleConfig};

struct SimpleHtml;

impl PosticleConfig for SimpleHtml {
    fn html_sanitizer(&self) -> Builder {
        let mut sanitizer = Builder::default();

        sanitizer.tags(hashset!["a", "br"]);

        sanitizer
    }

    fn transform_token(&self, token: Token) -> Vec<Token> {
        match token {
            Token::Link(link) => {
                let url = Url::parse(&link.0).unwrap();

                vec![Token::Element(Element(
                    "a".to_string(),
                    Some(vec![("href".to_string(), link.0.clone())]),
                    Some(url.domain().unwrap().to_string()),
                ))]
            },
            _ => vec![token],
        }
    }
}

fn main() {
    let posticle = Posticle::from(SimpleHtml);

    println!(
        "{}",
        posticle.render("Mastodon is great! https://joinmastodon.org/")
    )
}
