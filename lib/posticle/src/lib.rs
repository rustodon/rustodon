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
use tokens::*;

pub trait PosticleConfig {
    fn html_sanitizer(&self) -> Ammonia;
    fn transform_token(&self, token: Token) -> Vec<Token>;
}

struct DefaultConfig;

impl PosticleConfig for DefaultConfig {
    fn html_sanitizer(&self) -> Ammonia {
        let mut sanitizer = Ammonia::default();

        sanitizer.tags(hashset!["br"]);

        sanitizer
    }

    fn transform_token(&self, token: Token) -> Vec<Token> {
        vec![token]
    }
}

pub struct Posticle<'t> {
    config: Box<PosticleConfig + 't>,
}

impl<'t> Posticle<'t> {
    pub fn new() -> Self {
        Self {
            config: Box::new(DefaultConfig),
        }
    }

    // Given `text`, render as HTML.
    pub fn render(&self, text: &str) -> String {
        let config = &self.config;
        let mut output = String::new();
        let tokens = &self.parse(text);

        for token in tokens {
            match token {
                Token::Emoticon(token) => {
                    token.render(&mut output);
                },
                Token::Hashtag(token) => {
                    token.render(&mut output);
                },
                Token::LineBreak(token) => {
                    token.render(&mut output);
                },
                Token::Link(token) => {
                    token.render(&mut output);
                },
                Token::Mention(token) => {
                    token.render(&mut output);
                },
                Token::Text(token) => {
                    token.render(&mut output);
                },
                Token::Element(token) => {
                    token.render(&mut output);
                },
            }
        }

        config.html_sanitizer().clean(&output).to_string()
    }

    /// Given `text`, build an abstract syntax tree.
    pub fn parse(&self, text: &str) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();

        if let Ok(pairs) = document(text) {
            for pair in pairs {
                tokens.append(&mut Token::from_parse_pair(pair));
            }
        }

        self.transform_tokens(tokens)
    }

    /// The parser has a tendency to produce rows of text tokens, combine any text token that follows another text token into a new text token.
    fn normalize_text_tokens(&self, input: Vec<Token>) -> Vec<Token> {
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

    fn transform_tokens(&self, input: Vec<Token>) -> Vec<Token> {
        let config = &self.config;
        let mut output = Vec::new();

        for token in input {
            output.append(&mut config.transform_token(token));
        }

        self.normalize_text_tokens(output)
    }
}

impl<'t, T> From<T> for Posticle<'t>
where
    T: PosticleConfig + 't,
{
    fn from(config: T) -> Self {
        Self {
            config: Box::new(config),
        }
    }
}
