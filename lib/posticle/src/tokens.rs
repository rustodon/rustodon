use grammar::Rule;
use pest::iterators::Pair;

#[derive(Clone, Debug, PartialEq)]
pub struct Emoticon(pub String);

#[derive(Clone, Debug, PartialEq)]
pub struct Hashtag(pub String);

#[derive(Clone, Debug, PartialEq)]
pub struct Link(pub String, pub String);

#[derive(Clone, Debug, PartialEq)]
pub struct Mention(pub String, pub Option<String>);

#[derive(Clone, Debug, PartialEq)]
pub struct Text(pub String);

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Emoticon(Emoticon),
    Hashtag(Hashtag),
    LineBreak,
    Link(Link),
    Mention(Mention),
    Text(Text),
}

impl Token {
    pub fn from_parse_pair(pair: Pair<Rule>) -> Vec<Self> {
        match pair.as_rule() {
            Rule::emoticon => Self::from_emoticon_rule(pair),
            Rule::hashtag => Self::from_hashtag_rule(pair),
            Rule::line_break => vec![Token::LineBreak],
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
                    tokens.push(Token::Text(Text(pair.as_str().to_string())));
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
                    tokens.push(Token::Text(Text(pair.as_str().to_string())));
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
                    tokens.push(Token::Text(Text(pair.as_str().to_string())));
                },
            }
        }

        if let (Some(schema), Some(tail)) = (schema, tail) {
            let href = format!("{}{}", schema, tail);

            tokens.push(Token::Link(Link(tail, href)));
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
                    tokens.push(Token::Text(Text(pair.as_str().to_string())));
                },
            }
        }

        if let Some(username) = username {
            tokens.push(Token::Mention(Mention(username, domain)));
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
