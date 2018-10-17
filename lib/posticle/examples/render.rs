extern crate ammonia;
#[macro_use]
extern crate maplit;
extern crate posticle;

use ammonia::{Builder, Url};
use posticle::tokens::*;
use posticle::{Reader, Writer};

fn transform(token: Token) -> Token {
    match token {
        Token::Link(link) => {
            let url = Url::parse(&link.0).unwrap();

            Token::Element(Element(
                "a".to_string(),
                Some(vec![("href".to_string(), link.0.clone())]),
                Some(url.domain().unwrap().to_string()),
            ))
        },
        _ => token,
    }
}

fn main() {
    let mut html_sanitizer = Builder::default();

    html_sanitizer.tags(hashset!["br", "a"]);

    let reader = Reader::from("Mastodon is great! https://joinmastodon.org/");
    let mut writer = Writer::new().with_html_sanitizer(html_sanitizer);

    for token in reader {
        writer.push(transform(token));
    }

    println!("{}", writer.to_string());
}
