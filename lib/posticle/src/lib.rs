//! Posticle is a parser and renderer for Twitter and Mastodon like text.

#[macro_use]
extern crate pest_derive;

pub mod grammar;
pub mod tokens;

use crate::grammar::document;
use crate::tokens::*;
use ammonia::Builder as Ammonia;
use maplit::hashset;

/// Build a new [`Reader`].
pub struct ReaderBuilder<'t>(Reader<'t>);

impl Default for ReaderBuilder<'_> {
    fn default() -> Self {
        ReaderBuilder(Reader::default())
    }
}

impl<'t> ReaderBuilder<'t> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Finish building a [`Reader`] and tokenize its input.
    ///
    /// ```
    /// use posticle::ReaderBuilder;
    ///
    /// let reader = ReaderBuilder::new().with_str("Nice!").finish();
    ///
    /// assert_ne!(reader.into_vec(), Vec::new());
    /// ```
    pub fn finish(self) -> Reader<'t> {
        self.0.finish()
    }

    /// Add a [`str`] as input to the [`Reader`] being built.
    ///
    /// ```
    /// use posticle::{Reader, ReaderBuilder};
    ///
    /// let reader_a = ReaderBuilder::new().with_str("Hello world!").finish();
    /// let reader_b = Reader::from("Hello world!");
    ///
    /// assert_eq!(reader_a.into_vec(), reader_b.into_vec());
    /// ```
    pub fn with_str(self, input: &str) -> Self {
        self.with_string(input.to_string())
    }

    /// Add a [`String`] as input to the [`Reader`] being built.
    ///
    /// ```
    /// use posticle::{Reader, ReaderBuilder};
    ///
    /// let reader_a = ReaderBuilder::new()
    ///     .with_string(String::from("Hello world!"))
    ///     .finish();
    /// let reader_b = Reader::from("Hello world!");
    ///
    /// assert_eq!(reader_a.into_vec(), reader_b.into_vec());
    /// ```
    pub fn with_string(self, input: String) -> Self {
        ReaderBuilder(Reader { input, ..self.0 })
    }

    /// Add a transformer closure to the [`Reader`] being built.
    ///
    /// ```
    /// use posticle::tokens::*;
    /// use posticle::ReaderBuilder;
    ///
    /// let reader = ReaderBuilder::new()
    ///     .with_transformer(Box::new(|token| token))
    ///     .with_str("Hello world!")
    ///     .finish();
    ///
    /// assert_eq!(
    ///     reader.into_vec(),
    ///     vec![Token::Text(Text {
    ///         text: "Hello world!".to_string(),
    ///     })]
    /// );
    /// ```
    pub fn with_transformer(self, transformer: Box<dyn 't + Fn(Token) -> Token>) -> Self {
        ReaderBuilder(Reader {
            transformer,
            ..self.0
        })
    }
}

/// Read [`Token`]s from a string.
pub struct Reader<'t> {
    input: String,
    tokens: Vec<Token>,
    current_token: usize,
    transformer: Box<dyn 't + Fn(Token) -> Token>,
}

impl Default for Reader<'_> {
    fn default() -> Self {
        Reader {
            input: String::new(),
            tokens: Vec::new(),
            current_token: 0,
            transformer: Box::new(|token| token),
        }
    }
}

impl Iterator for Reader<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let current_token = self.current_token.to_owned();

        if current_token < self.tokens.len() {
            self.current_token += 1;

            if let Some(token) = self.tokens.get(current_token) {
                return Some(token.to_owned());
            }
        }

        None
    }
}

impl<'a, 't> From<&'a str> for Reader<'t> {
    /// ```
    /// use posticle::tokens::*;
    /// use posticle::Reader;
    ///
    /// let reader = Reader::from("Nice!");
    ///
    /// assert_eq!(
    ///     reader.into_vec(),
    ///     vec![Token::Text(Text {
    ///         text: "Nice!".to_string(),
    ///     })]
    /// );
    /// ```
    fn from(input: &str) -> Self {
        ReaderBuilder::new().with_str(input).finish()
    }
}

impl From<String> for Reader<'_> {
    /// ```
    /// use posticle::tokens::*;
    /// use posticle::Reader;
    ///
    /// let reader = Reader::from("Nice!".to_string());
    ///
    /// assert_eq!(
    ///     reader.into_vec(),
    ///     vec![Token::Text(Text {
    ///         text: "Nice!".to_string(),
    ///     })]
    /// );
    /// ```
    fn from(input: String) -> Self {
        ReaderBuilder::new().with_string(input).finish()
    }
}

impl From<Vec<Token>> for Reader<'_> {
    /// ```
    /// use posticle::tokens::*;
    /// use posticle::Reader;
    ///
    /// let reader = Reader::from(vec![Token::Text(Text {
    ///     text: "Nice!".to_string(),
    /// })]);
    ///
    /// assert_eq!(
    ///     reader.into_vec(),
    ///     vec![Token::Text(Text {
    ///         text: "Nice!".to_string(),
    ///     })]
    /// );
    /// ```
    fn from(tokens: Vec<Token>) -> Self {
        Reader {
            tokens,
            ..Self::default()
        }
    }
}

impl<'ta, 'tb> PartialEq<Reader<'ta>> for Reader<'tb> {
    fn eq(&self, other: &Reader) -> bool {
        self.input == other.input && self.tokens == other.tokens
    }
}

impl Reader<'_> {
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
    pub fn into_vec(self) -> Vec<Token> {
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
            Token::Text(Text { text }) => {
                replacement.push_str(&text);
            },
            _ => {
                if !replacement.is_empty() {
                    output.push(Token::Text(Text { text: replacement }));
                    replacement = String::new();
                }

                output.push(token);
            },
        }
    }

    if !replacement.is_empty() {
        output.push(Token::Text(Text { text: replacement }));
    }

    output
}

pub struct WriterBuilder<'w>(Writer<'w>);

impl Default for WriterBuilder<'_> {
    fn default() -> Self {
        WriterBuilder(Writer::default())
    }
}

impl<'w> WriterBuilder<'w> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Finish building a [`Writer`].
    ///
    /// ```
    /// extern crate ammonia;
    /// extern crate posticle;
    ///
    /// use ammonia::Builder;
    /// use posticle::WriterBuilder;
    ///
    /// let html_sanitizer = Builder::new();
    ///
    /// // html_sanitizer.tags(hashset!["a", "br"]);
    ///
    /// let writer = WriterBuilder::new().with_html_sanitizer(html_sanitizer);
    /// ```
    pub fn finish(self) -> Writer<'w> {
        self.0.finish()
    }

    /// Add a [`Reader`] as input to the [`Writer`] being built.
    ///
    /// ```
    /// use posticle::{Reader, WriterBuilder};
    ///
    /// let reader = Reader::from("Nice!");
    /// let writer = WriterBuilder::new().with_reader(reader).finish();
    ///
    /// assert_eq!(writer.to_string(), "Nice!".to_string());
    /// ```
    pub fn with_reader(self, reader: Reader) -> Self {
        self.with_tokens(reader.into_vec())
    }

    /// Add a [`Vec`] of [`Token`] as input to the [`Writer`] being built.
    ///
    /// ```
    /// use posticle::tokens::*;
    /// use posticle::{Reader, WriterBuilder};
    ///
    /// let tokens = vec![Token::Text(Text {
    ///     text: "Nice!".to_string(),
    /// })];
    /// let writer = WriterBuilder::new().with_tokens(tokens).finish();
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
    /// use posticle::{Reader, WriterBuilder};
    ///
    /// let tokens = vec![Token::Element(Element {
    ///     name: "x".to_string(),
    ///     attributes: Vec::new(),
    ///     children: vec![Token::Text(Text {
    ///         text: "Nice!".to_string(),
    ///     })],
    /// })];
    /// let writer = WriterBuilder::new()
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
    tokens: Vec<Token>,
}

impl Default for Writer<'_> {
    fn default() -> Self {
        let mut html_sanitizer = Ammonia::default();

        html_sanitizer.tags(hashset!["br"]);

        Self {
            output: String::new(),
            html_sanitizer,
            tokens: Vec::new(),
        }
    }
}

impl<'w, 't> From<Reader<'t>> for Writer<'w> {
    /// ```
    /// use posticle::tokens::*;
    /// use posticle::{Reader, Writer};
    ///
    /// let reader = Reader::from(vec![Token::Text(Text {
    ///     text: "Nice!".to_string(),
    /// })]);
    /// let writer = Writer::from(reader);
    ///
    /// assert_eq!(writer.to_string(), "Nice!".to_string());
    /// ```
    fn from(reader: Reader) -> Self {
        WriterBuilder::new().with_reader(reader).finish()
    }
}

impl From<Vec<Token>> for Writer<'_> {
    /// ```
    /// use posticle::tokens::*;
    /// use posticle::Writer;
    ///
    /// let writer = Writer::from(vec![Token::Text(Text {
    ///     text: "Nice!".to_string(),
    /// })]);
    ///
    /// assert_eq!(writer.to_string(), "Nice!".to_string());
    /// ```
    fn from(tokens: Vec<Token>) -> Self {
        WriterBuilder::new().with_tokens(tokens).finish()
    }
}

impl Writer<'_> {
    fn finish(mut self) -> Self {
        for token in self.tokens.to_owned() {
            self.push(&token);
        }

        self
    }

    /// Convert the [`Writer`] to a [`String`].
    pub fn to_string(&self) -> String {
        self.html_sanitizer.clean(&self.output).to_string()
    }

    /// Push a [`Token`] onto the [`Writer`].
    pub fn push(&mut self, token: &Token) {
        token.render(&mut self.output);
    }
}
