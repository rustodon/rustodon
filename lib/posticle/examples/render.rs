extern crate ammonia;
#[macro_use]
extern crate maplit;
extern crate posticle;

use ammonia::{Builder, Url};
use posticle::tokens::*;
use posticle::{Reader, Writer};

fn main() {
    let reader = Reader::new()
        .with_str("Mastodon is great! https://joinmastodon.org/")
        .with_transformer(Box::new(|token| match token {
            Token::Link(link) => {
                let url = Url::parse(&link.url).unwrap();

                Token::Element(Element {
                    name: "a".to_string(),
                    attributes: vec![("href".to_string(), link.url.clone())],
                    children: vec![Token::Text(Text {
                        text: url.domain().unwrap().to_string(),
                    })],
                })
            },
            _ => token,
        }))
        .finish();

    let mut html_sanitizer = Builder::default();

    html_sanitizer.tags(hashset!["br", "a"]);

    let html = Writer::new()
        .with_reader(reader)
        .with_html_sanitizer(html_sanitizer)
        .finish()
        .to_string();

    println!("{}", html);
}
