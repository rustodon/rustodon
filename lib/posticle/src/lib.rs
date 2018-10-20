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
use tokens::*;

/// Build a new [`Reader`].
pub struct ReaderBuilder(Reader);

impl Default for ReaderBuilder {
    fn default() -> Self {
        ReaderBuilder(Reader::default())
    }
}

impl ReaderBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Finish building a [`Reader`] and tokenize its input.
    ///
    /// ```
    /// use posticle::Reader;
    ///
    /// let reader = Reader::new().with_str("Nice!").finish();
    ///
    /// assert_ne!(reader.to_vec(), Vec::new());
    /// ```
    pub fn finish(self) -> Reader {
        self.0.finish()
    }

    /// Add a [`str`] as input to the [`Reader`] being built.
    ///
    /// ```
    /// use posticle::Reader;
    ///
    /// let reader_a = Reader::new().with_str("Hello world!").finish();
    /// let reader_b = Reader::from("Hello world!");
    ///
    /// assert_eq!(reader_a.to_vec(), reader_b.to_vec());
    /// ```
    pub fn with_str(self, input: &str) -> Self {
        self.with_string(input.to_string())
    }

    /// Add a [`String`] as input to the [`Reader`] being built.
    ///
    /// ```
    /// use posticle::Reader;
    ///
    /// let reader_a = Reader::new()
    ///     .with_string(String::from("Hello world!"))
    ///     .finish();
    /// let reader_b = Reader::from("Hello world!");
    ///
    /// assert_eq!(reader_a.to_vec(), reader_b.to_vec());
    /// ```
    pub fn with_string(self, input: String) -> Self {
        ReaderBuilder(Reader { input, ..self.0 })
    }

    /// Add a transformer closure to the [`Reader`] being built.
    ///
    /// ```
    /// use posticle::tokens::*;
    /// use posticle::Reader;
    ///
    /// let reader = Reader::new()
    ///     .with_transformer(Box::new(|token| token))
    ///     .with_str("Hello world!")
    ///     .finish();
    ///
    /// assert_eq!(
    ///     reader.to_vec(),
    ///     vec![Token::Text(Text("Hello world!".to_string()))]
    /// );
    /// ```
    pub fn with_transformer(self, transformer: Box<Fn(Token) -> Token>) -> Self {
        ReaderBuilder(Reader {
            transformer,
            ..self.0
        })
    }
}

/// Read [`Token`]s from a string.
pub struct Reader {
    input: String,
    tokens: Vec<Token>,
    current_token: usize,
    transformer: Box<Fn(Token) -> Token>,
}

impl Default for Reader {
    fn default() -> Self {
        Reader {
            input: String::new(),
            tokens: Vec::new(),
            current_token: 0,
            transformer: Box::new(|token| token),
        }
    }
}

impl Iterator for Reader {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let current_token = self.current_token.to_owned();

        if current_token < self.tokens.len() {
            self.current_token = self.current_token + 1;

            if let Some(token) = self.tokens.get(current_token) {
                return Some(token.to_owned());
            }
        }

        None
    }
}

impl<'a> From<&'a str> for Reader {
    /// ```
    /// use posticle::tokens::*;
    /// use posticle::Reader;
    ///
    /// let reader = Reader::from("Nice!");
    ///
    /// assert_eq!(
    ///     reader.to_vec(),
    ///     vec![Token::Text(Text("Nice!".to_string()))]
    /// );
    /// ```
    fn from(input: &str) -> Self {
        ReaderBuilder::new().with_str(input).finish()
    }
}

impl From<String> for Reader {
    /// ```
    /// use posticle::tokens::*;
    /// use posticle::Reader;
    ///
    /// let reader = Reader::from("Nice!".to_string());
    ///
    /// assert_eq!(
    ///     reader.to_vec(),
    ///     vec![Token::Text(Text("Nice!".to_string()))]
    /// );
    /// ```
    fn from(input: String) -> Self {
        ReaderBuilder::new().with_string(input).finish()
    }
}

impl From<Vec<Token>> for Reader {
    /// ```
    /// use posticle::tokens::*;
    /// use posticle::Reader;
    ///
    /// let reader = Reader::from(vec![Token::Text(Text("Nice!".to_string()))]);
    ///
    /// assert_eq!(
    ///     reader.to_vec(),
    ///     vec![Token::Text(Text("Nice!".to_string()))]
    /// );
    /// ```
    fn from(tokens: Vec<Token>) -> Self {
        Reader {
            tokens: tokens,
            ..Self::default()
        }
    }
}

impl PartialEq<Reader> for Reader {
    fn eq(&self, other: &Reader) -> bool {
        self.input == other.input && self.tokens == other.tokens
    }

    fn ne(&self, other: &Reader) -> bool {
        self.input != other.input || self.tokens != other.tokens
    }
}

impl Reader {
    /// Build a new [`Reader`].
    pub fn new() -> ReaderBuilder {
        ReaderBuilder::new()
    }

    fn finish(self) -> Self {
        let mut tokens: Vec<Token> = Vec::new();

        if let Ok(pairs) = document(&self.input) {
            let transformer = &self.transformer;

            for pair in pairs {
                tokens.append(&mut Token::from_parse_pair(pair, transformer));
            }
        }

        let tokens = normalize_text_tokens(tokens);

        Self { tokens, ..self }
    }

    /// Convert a [`Reader`] to a [`Vec`] of [`Token`].
    pub fn to_vec(self) -> Vec<Token> {
        self.tokens
    }
}

/// The parser has a tendency to produce rows of text tokens, combine any
/// text token that follows another text token into a new text token.
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

pub struct WriterBuilder<'w>(Writer<'w>);

impl<'w> Default for WriterBuilder<'w> {
    fn default() -> Self {
        WriterBuilder(Writer::default())
    }
}

impl<'w> WriterBuilder<'w> {
    /// Finish building a [`Writer`].
    ///
    /// ```
    /// extern crate ammonia;
    /// extern crate posticle;
    ///
    /// use ammonia::Builder;
    /// use posticle::Writer;
    ///
    /// let html_sanitizer = Builder::new();
    ///
    /// // html_sanitizer.tags(hashset!["a", "br"]);
    ///
    /// let writer = Writer::new().with_html_sanitizer(html_sanitizer);
    /// ```
    pub fn finish(self) -> Writer<'w> {
        self.0.finish()
    }

    /// Add a [`Reader`] as input to the [`Writer`] being built.
    ///
    /// ```
    /// use posticle::{Reader, Writer};
    ///
    /// let reader = Reader::from("Nice!");
    /// let writer = Writer::new().with_reader(reader).finish();
    ///
    /// assert_eq!(writer.to_string(), "Nice!".to_string());
    /// ```
    pub fn with_reader(self, reader: Reader) -> Self {
        self.with_tokens(reader.to_vec())
    }

    /// Add a [`Vec`] of [`Token`] as input to the [`Writer`] being built.
    ///
    /// ```
    /// use posticle::tokens::*;
    /// use posticle::{Reader, Writer};
    ///
    /// let tokens = vec![Token::Text(Text("Nice!".to_string()))];
    /// let writer = Writer::new().with_tokens(tokens).finish();
    ///
    /// assert_eq!(writer.to_string(), "Nice!".to_string());
    /// ```
    pub fn with_tokens(self, tokens: Vec<Token>) -> Self {
        WriterBuilder(Writer { tokens, ..self.0 })
    }

    /// Add an [`ammonia::Builder`] to the [`Writer`] being built.
    ///
    /// ```
    /// extern crate ammonia;
    /// extern crate posticle;
    ///
    /// use ammonia::Builder;
    /// use posticle::tokens::*;
    /// use posticle::{Reader, Writer};
    ///
    /// let tokens = vec![Token::Element(Element(
    ///     "x".to_string(),
    ///     None,
    ///     Some("Nice!".to_string()),
    /// ))];
    /// let writer = Writer::new()
    ///     .with_tokens(tokens)
    ///     .with_html_sanitizer(Builder::new())
    ///     .finish();
    ///
    /// // The <x> and </x> tags will be stripped.
    /// assert_eq!(writer.to_string(), "Nice!".to_string());
    /// ```
    pub fn with_html_sanitizer(self, html_sanitizer: Ammonia<'w>) -> Self {
        WriterBuilder(Writer {
            html_sanitizer,
            ..self.0
        })
    }
}

/// Write [`Token`]s as HTML to a string.
pub struct Writer<'w> {
    output: String,
    html_sanitizer: Ammonia<'w>,
    html_tags: Vec<String>,
    tokens: Vec<Token>,
}

impl<'w> Default for Writer<'w> {
    fn default() -> Self {
        let mut html_sanitizer = Ammonia::default();

        html_sanitizer.tags(hashset!["br"]);

        Self {
            output: String::new(),
            html_tags: vec!["br".to_string()],
            html_sanitizer,
            tokens: Vec::new(),
        }
    }
}

impl<'w> From<Reader> for Writer<'w> {
    /// ```
    /// use posticle::tokens::*;
    /// use posticle::{Reader, Writer};
    ///
    /// let reader = Reader::from(vec![Token::Text(Text("Nice!".to_string()))]);
    /// let writer = Writer::from(reader);
    ///
    /// assert_eq!(writer.to_string(), "Nice!".to_string());
    /// ```
    fn from(reader: Reader) -> Self {
        Writer::new().with_reader(reader).finish()
    }
}

impl<'w> From<Vec<Token>> for Writer<'w> {
    /// ```
    /// use posticle::tokens::*;
    /// use posticle::Writer;
    ///
    /// let writer = Writer::from(vec![Token::Text(Text("Nice!".to_string()))]);
    ///
    /// assert_eq!(writer.to_string(), "Nice!".to_string());
    /// ```
    fn from(tokens: Vec<Token>) -> Self {
        Writer::new().with_tokens(tokens).finish()
    }
}

impl<'w> Writer<'w> {
    /// Build a new [`Writer`].
    pub fn new() -> WriterBuilder<'w> {
        WriterBuilder::default()
    }

    fn finish(mut self) -> Self {
        let tokens = &self.tokens.to_owned();

        for token in tokens {
            self.push(token.to_owned());
        }

        self
    }

    /// Convert the [`Writer`] to a [`String`].
    pub fn to_string(self) -> String {
        self.html_sanitizer.clean(&self.output).to_string()
    }

    /// Push a [`Token`] onto the [`Writer`].
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
