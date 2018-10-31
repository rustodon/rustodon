use grammar::Rule;
use pest::iterators::Pair;

fn html_escape(text: &String) -> String {
    text.replace('&', "&amp;")
        .replace('"', "&quot")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[derive(Clone, Debug, PartialEq)]
/// A textual emoticon.
pub struct Emoticon {
    pub name: String,
}

impl Emoticon {
    pub fn render(&self, output: &mut String) {
        output.push_str(&format!(":{}:", html_escape(&self.name)));
    }
}

#[derive(Clone, Debug, PartialEq)]
/// A hashtag.
pub struct Hashtag {
    pub name: String,
}

impl Hashtag {
    pub fn render(&self, output: &mut String) {
        output.push_str(&format!("#{}", html_escape(&self.name)));
    }
}

#[derive(Clone, Debug, PartialEq)]
/// A line break.
pub struct LineBreak;

impl LineBreak {
    pub fn render(&self, output: &mut String) {
        output.push_str("\n<br>");
    }
}

#[derive(Clone, Debug, PartialEq)]
/// A link to a resource with text and href.
pub struct Link {
    pub url: String,
}

impl Link {
    pub fn render(&self, output: &mut String) {
        output.push_str(&html_escape(&self.url));
    }
}

#[derive(Clone, Debug, PartialEq)]
/// A mention with an optional domain.
pub struct Mention {
    pub username: String,
    pub domain:   Option<String>,
}

impl Mention {
    pub fn render(&self, output: &mut String) {
        output.push_str(&format!("@{}", html_escape(&self.username)));

        if let Some(domain) = &self.domain {
            output.push_str(&format!("@{}", html_escape(domain)));
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
/// Plain text that will have its entities encoded on render.
pub struct Text {
    pub text: String,
}

impl Text {
    pub fn render(&self, output: &mut String) {
        output.push_str(&html_escape(&self.text));
    }
}

#[derive(Clone, Debug, PartialEq)]
/// Trusted HTML element with content that will have its entities encoded on render.
pub struct Element {
    pub name: String,
    pub attributes: Vec<(String, String)>,
    pub children: Vec<Token>,
}

impl Element {
    pub fn render(&self, output: &mut String) {
        output.push_str("<");
        output.push_str(&self.name);

        if !self.attributes.is_empty() {
            for (name, value) in &self.attributes {
                output.push_str(" ");
                output.push_str(name);
                output.push_str("=\"");
                output.push_str(&html_escape(value));
                output.push_str("\"");
            }
        }

        output.push_str(">");

        if !self.children.is_empty() {
            for child in &self.children {
                child.render(output);
            }
        }

        output.push_str("</");
        output.push_str(&self.name);
        output.push_str(">");
    }
}

#[derive(Clone, Debug, PartialEq)]
/// An item in the abstract syntax tree.
pub enum Token {
    Emoticon(Emoticon),
    Hashtag(Hashtag),
    LineBreak(LineBreak),
    Link(Link),
    Mention(Mention),
    Text(Text),
    Element(Element),
}

impl Token {
    pub fn from_parse_pair<'t>(pair: Pair<Rule>, transformer: &Box<'t + Fn(Token) -> Token>) -> Vec<Self> {
        match pair.as_rule() {
            Rule::emoticon => Self::from_emoticon_rule(pair, transformer),
            Rule::hashtag => Self::from_hashtag_rule(pair, transformer),
            Rule::line_break => vec![transformer(Token::LineBreak(LineBreak))],
            Rule::link => Self::from_link_rule(pair, transformer),
            Rule::mention => Self::from_mention_rule(pair, transformer),
            _ => vec![transformer(Token::Text(Text {
                text: pair.as_str().to_string(),
            }))],
        }
    }

    fn from_emoticon_rule<'t>(pair: Pair<Rule>, transformer: &Box<'t + Fn(Token) -> Token>) -> Vec<Self> {
        let mut tokens = Vec::new();
        let mut name: Option<String> = None;

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::emoticon_name => {
                    name = Some(pair.as_str().to_string());
                },
                _ => {
                    tokens.append(&mut Self::from_symbol_prefix(pair, transformer));
                },
            }
        }

        if let Some(name) = name {
            tokens.push(transformer(Token::Emoticon(Emoticon { name })));
        }

        tokens
    }

    fn from_hashtag_rule<'t>(pair: Pair<Rule>, transformer: &Box<'t + Fn(Token) -> Token>) -> Vec<Self> {
        let mut tokens = Vec::new();
        let mut name: Option<String> = None;

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::hashtag_name => {
                    name = Some(pair.as_str().to_string());
                },
                _ => {
                    tokens.append(&mut Self::from_symbol_prefix(pair, transformer));
                },
            }
        }

        if let Some(name) = name {
            tokens.push(transformer(Token::Hashtag(Hashtag { name })));
        }

        tokens
    }

    fn from_link_rule<'t>(pair: Pair<Rule>, transformer: &Box<'t + Fn(Token) -> Token>) -> Vec<Self> {
        let mut tokens = Vec::new();
        let mut schema: Option<String> = None;
        let mut tail: Option<String> = None;

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::link_schema => {
                    schema = Some(pair.as_str().to_string());
                },
                Rule::link_tail => {
                    tail = Some(pair.as_str().to_string());
                },
                _ => {
                    tokens.append(&mut Self::from_symbol_prefix(pair, transformer));
                },
            }
        }

        if let (Some(schema), Some(tail)) = (schema, tail) {
            let url = format!("{}{}", schema, tail);

            tokens.push(transformer(Token::Link(Link { url })));
        }

        tokens
    }

    fn from_mention_rule<'t>(pair: Pair<Rule>, transformer: &Box<'t + Fn(Token) -> Token>) -> Vec<Self> {
        let mut tokens = Vec::new();
        let mut username: Option<String> = None;
        let mut domain: Option<String> = None;

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::mention_username => {
                    username = Some(pair.as_str().to_string());
                },
                Rule::mention_domain => {
                    domain = Some(pair.as_str().to_string());
                },
                _ => {
                    tokens.append(&mut Self::from_symbol_prefix(pair, transformer));
                },
            }
        }

        if let Some(username) = username {
            tokens.push(transformer(Token::Mention(Mention { username, domain })));
        }

        tokens
    }

    fn from_symbol_prefix<'t>(pair: Pair<Rule>, transformer: &Box<'t + Fn(Token) -> Token>) -> Vec<Self> {
        let mut tokens = Vec::new();

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::line_break => {
                    tokens.push(transformer(Token::LineBreak(LineBreak)));
                },
                _ => {
                    let text = pair.as_str().to_string();

                    tokens.push(transformer(Token::Text(Text { text })));
                },
            }
        }

        tokens
    }

    pub fn render(&self, output: &mut String) {
        match self {
            Token::Emoticon(token) => {
                token.render(output);
            },
            Token::Hashtag(token) => {
                token.render(output);
            },
            Token::LineBreak(token) => {
                token.render(output);
            },
            Token::Link(token) => {
                token.render(output);
            },
            Token::Mention(token) => {
                token.render(output);
            },
            Token::Text(token) => {
                token.render(output);
            },
            Token::Element(token) => {
                token.render(output);
            },
        }
    }
}
