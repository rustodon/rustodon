//! Posticle is a parser and renderer for Twitter and Mastodon like text.

#![feature(nll)]

extern crate ammonia;
#[macro_use]
extern crate maplit;
extern crate pest;
#[macro_use]
extern crate pest_derive;

pub mod grammar;
pub mod tokens;

use ammonia::Builder as Ammonia;
use grammar::document;
use std::vec::IntoIter;
use tokens::*;

/// An iteratable reader of `posticle::tokens::Token`.
pub struct Reader {
    tokens: IntoIter<Token>,
}

impl Default for Reader {
    fn default() -> Self {
        Reader {
            tokens: Vec::new().into_iter(),
        }
    }
}

impl Iterator for Reader {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.tokens.next()
    }
}

impl<'a> From<&'a str> for Reader {
    fn from(text: &str) -> Reader {
        let mut tokens: Vec<Token> = Vec::new();

        if let Ok(pairs) = document(text) {
            for pair in pairs {
                tokens.append(&mut Token::from_parse_pair(pair));
            }
        }

        Reader {
            tokens: normalize_text_tokens(tokens).into_iter(),
        }
    }
}

impl From<String> for Reader {
    fn from(text: String) -> Reader {
        Self::from(text.as_str())
    }
}

impl Reader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn to_vec(self) -> Vec<Token> {
        self.tokens.collect()
    }
}

/// The parser has a tendency to produce rows of text tokens, combine any text token that follows another text token into a new text token.
fn normalize_text_tokens(input: Vec<Token>) -> Vec<Token> {
    let mut output = Vec::new();
    let mut replacement = String::new();

    for token in input {
        match token {
            Token::Text(Text(text)) => {
                replacement.push_str(&text);
            },
            _ => {
                if replacement.len() > 0 {
                    output.push(Token::Text(Text(replacement)));
                    replacement = String::new();
                }

                output.push(token);
            },
        }
    }

    if replacement.len() > 0 {
        output.push(Token::Text(Text(replacement)));
    }

    output
}

/// Write `posticle::tokens::Token`s to a String as HTML.
pub struct Writer<'w> {
    output: String,
    html_sanitizer: Ammonia<'w>,
    html_tags: Vec<String>,
}

impl<'w> Default for Writer<'w> {
    fn default() -> Self {
        let mut html_sanitizer = Ammonia::default();

        html_sanitizer.tags(hashset!["br"]);

        Self {
            output: String::new(),
            html_tags: vec!["br".to_string()],
            html_sanitizer,
        }
    }
}

impl<'w> From<Reader> for Writer<'w> {
    fn from(reader: Reader) -> Writer<'w> {
        let mut writer = Self::default();

        for token in reader {
            writer.push(token);
        }

        writer
    }
}

impl<'w> Writer<'w> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_html_sanitizer(self, html_sanitizer: Ammonia<'w>) -> Self {
        Self {
            html_sanitizer,
            ..self
        }
    }

    pub fn to_string(self) -> String {
        self.html_sanitizer.clean(&self.output).to_string()
    }

    pub fn push(&mut self, token: Token) {
        match token {
            Token::Emoticon(token) => {
                token.render(&mut self.output);
            },
            Token::Hashtag(token) => {
                token.render(&mut self.output);
            },
            Token::LineBreak(token) => {
                token.render(&mut self.output);
            },
            Token::Link(token) => {
                token.render(&mut self.output);
            },
            Token::Mention(token) => {
                token.render(&mut self.output);
            },
            Token::Text(token) => {
                token.render(&mut self.output);
            },
            Token::Element(token) => {
                let tag = token.0.clone();

                if !self.html_tags.contains(&tag) {
                    self.html_tags.push(tag);
                }

                token.render(&mut self.output);
            },
        }
    }
}
