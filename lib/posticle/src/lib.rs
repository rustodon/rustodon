#![feature(nll)]

#[macro_use]
extern crate lazy_static;
extern crate pest;
#[macro_use]
extern crate pest_derive;
#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;
extern crate regex;
extern crate validator;

mod grammar;

use grammar::{Grammar, Rule};
use pest::iterators::Pair;
use pest::Parser;
use regex::Regex;

lazy_static! {
    /// Matches all valid characters in a hashtag name (after the first #).
    static ref VALID_HASHTAG_NAME_RE: Regex = Regex::new(r"^[[:word:]_]*[[:alpha:]_·][[:word:]_]*$").unwrap();

    /// Matches all valid characters in a mention username (after the first @).
    static ref VALID_MENTION_USERNAME_RE: Regex = Regex::new(r"^[a-z0-9_]+([a-z0-9_\.]+[a-z0-9_]+)?$").unwrap();
}

#[derive(Debug, PartialEq, Eq, Hash)]
/// Tags a given [Entity]'s semantic kind.
pub enum EntityKind {
    /// A URL.
    Url,
    /// A hashtag.
    Hashtag,
    /// A mention; the inner data is `(username, optional domain)`.
    Mention(String, Option<String>),
}

#[derive(Debug, PartialEq, Eq, Hash)]
/// Represents an entity extracted from a given string of text.
///
/// The entity is described by its `kind` and the `range` of indices it occupies within the string.
pub struct Entity {
    pub kind:  EntityKind,
    pub range: (usize, usize),
}

impl Entity {
    /// Extracts the range described by this Entity from the passed `text`.
    pub fn substr<'a>(&self, text: &'a str) -> &'a str {
        &text[self.range.0..self.range.1]
    }

    /// Returns `true` if this entity overlaps with some `other` entity.
    pub fn overlaps_with(&self, other: &Entity) -> bool {
        self.range.0 <= other.range.1 && other.range.0 <= self.range.1
    }

    /// Create an Entity of `EntityKind::Hashtag` from parser result.
    pub fn from_hashtag(pair: Pair<Rule>) -> Option<Self> {
        let span = pair.as_span();
        let start = span.start_pos().pos();
        let end = span.end_pos().pos();
        let pairs = pair.into_inner();

        for pair in pairs {
            match pair.as_rule() {
                Rule::hashtag_name => {
                    if validate_hashtag_name(pair.as_str()) {
                        return Some(Entity {
                            kind:  EntityKind::Hashtag,
                            range: (start, end),
                        });
                    }
                },
                _ => unreachable!(),
            }
        }

        None
    }

    /// Create an Entity of `EntityKind::Mention` from parser result.
    pub fn from_mention(pair: Pair<Rule>) -> Option<Self> {
        let span = pair.as_span();
        let start = span.start_pos().pos();
        let end = span.end_pos().pos();
        let pairs = pair.into_inner();
        let mut username = None;
        let mut domain = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::mention_username => {
                    let value = pair.as_str();

                    if validate_mention_username(&value) {
                        username = Some(value.to_string());
                    }
                },
                Rule::mention_domain => {
                    let value = pair.as_str();

                    if validate_mention_domain(&value) {
                        domain = Some(value.to_string());
                    } else {
                        username = None;
                    }
                },
                _ => unreachable!(),
            }
        }

        if let Some(username) = username {
            Some(Entity {
                kind:  EntityKind::Mention(username, domain),
                range: (start, end),
            })
        } else {
            None
        }
    }

    /// Create an Entity of `EntityKind::Url` from parser result.
    pub fn from_url(pair: Pair<Rule>) -> Option<Self> {
        let span = pair.as_span();
        let start = span.start_pos().pos();
        let end = span.end_pos().pos();

        if validator::validate_url(pair.as_str()) {
            Some(Entity {
                kind:  EntityKind::Url,
                range: (start, end),
            })
        } else {
            None
        }
    }
}

/// Given `text`, extract all [Entities](Entity)
pub fn entities(text: &str) -> Vec<Entity> {
    let mut results = Vec::new();

    if let Ok(pairs) = Grammar::parse(Rule::post, text) {
        for pair in pairs {
            match match pair.as_rule() {
                Rule::hashtag => Entity::from_hashtag(pair),
                Rule::mention => Entity::from_mention(pair),
                Rule::url => Entity::from_url(pair),
                _ => unreachable!(),
            } {
                Some(entity) => results.push(entity),
                None => {},
            }
        }
    }

    results
}

/// Check that a hashtag name (after the first #) is valid.
fn validate_hashtag_name(name: &str) -> bool {
    VALID_HASHTAG_NAME_RE.is_match(name)
}

/// Check that a mentioned username is valid.
fn validate_mention_username(username: &str) -> bool {
    VALID_MENTION_USERNAME_RE.is_match(username)
}

/// Check that a mentioned instance domain is valid.
fn validate_mention_domain(domain: &str) -> bool {
    validator::validate_url(format!("https://{}", domain))
}

#[cfg(test)]
mod tests {
    extern crate yaml_rust;
    use super::*;

    const TLDS_YAML: &'static str = include_str!("../vendor/test/tlds.yml");

    #[test]
    fn extracts_nothing() {
        assert_eq!(entities("a string without at signs"), vec![]);
    }

    #[test]
    fn extracts_mentions() {
        assert_eq!(
            entities("@mention"),
            vec![Entity {
                kind:  EntityKind::Mention("mention".to_string(), None),
                range: (0, 8),
            }]
        );
        assert_eq!(
            entities("@mention@domain.place"),
            vec![Entity {
                kind:  EntityKind::Mention("mention".to_string(), Some("domain.place".to_string())),
                range: (0, 21),
            }]
        );
    }

    #[test]
    fn ignores_invalid_mentions() {
        assert_eq!(entities("@@yuser@domain"), vec![]);
        assert_eq!(entities("@xuser@@domain"), vec![]);
        assert_eq!(entities("@zuser@-domain-.com"), vec![]);
    }

    #[test]
    fn extracts_hashtags() {
        assert_eq!(
            entities("#hashtag"),
            vec![Entity {
                kind:  EntityKind::Hashtag,
                range: (0, 8),
            }]
        );
    }

    #[test]
    fn extracts_urls() {
        assert_eq!(
            entities("https://example.com"),
            vec![Entity {
                kind:  EntityKind::Url,
                range: (0, 19),
            }]
        );
    }

    #[test]
    fn extracts_all() {
        assert_eq!(
            entities("text #hashtag https://example.com @mention text"),
            vec![
                Entity {
                    kind:  EntityKind::Hashtag,
                    range: (5, 13),
                },
                Entity {
                    kind:  EntityKind::Url,
                    range: (14, 33),
                },
                Entity {
                    kind:  EntityKind::Mention("mention".to_string(), None),
                    range: (34, 42),
                },
            ]
        );
    }

    #[test]
    fn all_tlds_validate() {
        let tests = yaml_rust::YamlLoader::load_from_str(TLDS_YAML).unwrap();
        let tests = tests.first().unwrap();
        let ref tests = tests["tests"];

        for (suite, test_cases) in tests.as_hash().expect("could not load tests document") {
            let suite = suite.as_str().expect("suite could not be loaded");

            for test in test_cases.as_vec().expect("suite could not be loaded") {
                let description = test["description"]
                    .as_str()
                    .expect("test was missing 'description'");
                let text = test["text"].as_str().expect("test was missing 'text'");

                assert!(
                    validate_mention_domain(text),
                    "test {}/\"{}\" failed on text \"{}\"",
                    suite,
                    description,
                    text
                );
            }
        }
    }
}
