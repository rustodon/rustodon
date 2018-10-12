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
pub struct Emoticon(pub String);

impl Emoticon {
    pub fn render(&self, output: &mut String) {
        output.push_str(&format!(":{}:", html_escape(&self.0)));
    }
}

#[derive(Clone, Debug, PartialEq)]
/// A hashtag.
pub struct Hashtag(pub String);

impl Hashtag {
    pub fn render(&self, output: &mut String) {
        output.push_str(&format!("#{}", html_escape(&self.0)));
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
pub struct Link(pub String);

impl Link {
    pub fn render(&self, output: &mut String) {
        output.push_str(&html_escape(&self.0));
    }
}

#[derive(Clone, Debug, PartialEq)]
/// A mention with an optional domain.
pub struct Mention(pub String, pub Option<String>);

impl Mention {
    pub fn render(&self, output: &mut String) {
        if let Some(domain) = &self.1 {
            output.push_str(&format!(
                "@{}@{}",
                html_escape(&self.0),
                html_escape(domain)
            ));
        } else {
            output.push_str(&format!("@{}", html_escape(&self.0)));
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
/// Plain text that will have its entities encoded on render.
pub struct Text(pub String);

impl Text {
    pub fn render(&self, output: &mut String) {
        output.push_str(&html_escape(&self.0));
    }
}

#[derive(Clone, Debug, PartialEq)]
/// Untrusted HTML that will be sanitized on render.
pub struct Html(pub String);

impl Html {
    pub fn render(&self, output: &mut String) {
        output.push_str(&self.0);
    }
}

#[derive(Clone, Debug, PartialEq)]
/// Untrusted HTML that will be sanitized on render.
pub struct Element(
    pub String,
    pub Option<Vec<(String, String)>>,
    pub Option<String>,
);

impl Element {
    pub fn render(&self, output: &mut String) {
        output.push_str("<");
        output.push_str(&self.0);

        if let Some(attributes) = &self.1 {
            for (name, value) in attributes {
                output.push_str(" ");
                output.push_str(name);
                output.push_str("=\"");
                output.push_str(&html_escape(value));
                output.push_str("\"");
            }
        }

        output.push_str(">");

        if let Some(text) = &self.2 {
            output.push_str(&html_escape(text));
        }

        output.push_str("</");
        output.push_str(&self.0);
        output.push_str(">");
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Emoticon(Emoticon),
    Hashtag(Hashtag),
    LineBreak(LineBreak),
    Link(Link),
    Mention(Mention),
    Text(Text),
    Html(Html),
    Element(Element),
}

impl Token {
    pub fn from_parse_pair(pair: Pair<Rule>) -> Vec<Self> {
        match pair.as_rule() {
            Rule::emoticon => Self::from_emoticon_rule(pair),
            Rule::hashtag => Self::from_hashtag_rule(pair),
            Rule::line_break => vec![Token::LineBreak(LineBreak)],
            Rule::link => Self::from_link_rule(pair),
            Rule::mention => Self::from_mention_rule(pair),
            _ => vec![Token::Text(Text(pair.as_str().to_string()))],
        }
    }

    fn from_emoticon_rule(pair: Pair<Rule>) -> Vec<Self> {
        let mut tokens = Vec::new();
        let mut name: Option<String> = None;

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::emoticon_name => {
                    name = Some(pair.as_str().to_string());
                },
                _ => {
                    tokens.append(&mut Self::from_symbol_prefix(pair));
                },
            }
        }

        if let Some(name) = name {
            tokens.push(Token::Emoticon(Emoticon(name)));
        }

        tokens
    }

    fn from_hashtag_rule(pair: Pair<Rule>) -> Vec<Self> {
        let mut tokens = Vec::new();
        let mut name: Option<String> = None;

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::hashtag_name => {
                    name = Some(pair.as_str().to_string());
                },
                _ => {
                    tokens.append(&mut Self::from_symbol_prefix(pair));
                },
            }
        }

        if let Some(name) = name {
            tokens.push(Token::Hashtag(Hashtag(name)));
        }

        tokens
    }

    fn from_link_rule(pair: Pair<Rule>) -> Vec<Self> {
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
                    tokens.append(&mut Self::from_symbol_prefix(pair));
                },
            }
        }

        if let (Some(schema), Some(tail)) = (schema, tail) {
            tokens.push(Token::Link(Link(format!("{}{}", schema, tail))));
        }

        tokens
    }

    fn from_mention_rule(pair: Pair<Rule>) -> Vec<Self> {
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
                    tokens.append(&mut Self::from_symbol_prefix(pair));
                },
            }
        }

        if let Some(username) = username {
            tokens.push(Token::Mention(Mention(username, domain)));
        }

        tokens
    }

    fn from_symbol_prefix(pair: Pair<Rule>) -> Vec<Self> {
        let mut tokens = Vec::new();

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::line_break => {
                    tokens.push(Token::LineBreak(LineBreak));
                },
                _ => {
                    tokens.push(Token::Text(Text(pair.as_str().to_string())));
                },
            }
        }

        tokens
    }
}

pub trait TokenTransformer {
    fn transform(&self, token: Token) -> Vec<Token>;
}

pub struct DefaultTransformer;

impl TokenTransformer for DefaultTransformer {
    fn transform(&self, token: Token) -> Vec<Token> {
        vec![token]
    }
}
