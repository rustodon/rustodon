#![feature(nll)]

extern crate pest;
#[macro_use]
extern crate pest_derive;

// extern crate regex;
// #[macro_use]
// extern crate lazy_static;

#[macro_use]
#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

mod grammar;

use grammar::{Grammar, Rule};
use pest::iterators::Pair;
use pest::Parser;

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
    pub fn from_hashtag(pair: Pair<Rule>) -> Self {
        let span = pair.as_span();
        let start = span.start_pos().pos();
        let end = span.end_pos().pos();

        Entity {
            kind:  EntityKind::Hashtag,
            range: (start, end),
        }
    }

    /// Create an Entity of `EntityKind::Mention` from parser result.
    pub fn from_mention(pair: Pair<Rule>) -> Self {
        let span = pair.as_span();
        let start = span.start_pos().pos();
        let end = span.end_pos().pos();
        let pairs = pair.into_inner();
        let mut username = String::new();
        let mut domain = String::new();

        for pair in pairs {
            match pair.as_rule() {
                Rule::mention_username => {
                    username.push_str(pair.as_str());
                },
                Rule::mention_domain => {
                    domain.push_str(pair.as_str());
                },
                _ => unreachable!(),
            }
        }

        if domain.len() > 0 {
            Entity {
                kind:  EntityKind::Mention(username, Some(domain)),
                range: (start, end),
            }
        } else {
            Entity {
                kind:  EntityKind::Mention(username, None),
                range: (start, end),
            }
        }
    }

    /// Create an Entity of `EntityKind::Url` from parser result.
    pub fn from_url(pair: Pair<Rule>) -> Self {
        let span = pair.as_span();
        let start = span.start_pos().pos();
        let end = span.end_pos().pos();

        Entity {
            kind:  EntityKind::Url,
            range: (start, end),
        }
    }
}

/// Given `text`, extract all [Entities](Entity)
pub fn entities(text: &str) -> Vec<Entity> {
    let pairs = Grammar::parse(Rule::post, text).unwrap_or_else(|e| panic!("{}", e));
    let mut results = Vec::new();

    for pair in pairs {
        results.push(match pair.as_rule() {
            Rule::hashtag => Entity::from_hashtag(pair),
            Rule::mention => Entity::from_mention(pair),
            Rule::url => Entity::from_url(pair),
            _ => unreachable!(),
        });
    }

    results
}

#[cfg(test)]
mod tests {
    extern crate yaml_rust;
    use super::*;
    // use std::collections::HashSet;

    // const TLDS_YAML: &'static str = include_str!("../vendor/test/tlds.yml");

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
            entities("#hashtag https://example.com @mention"),
            vec![
                Entity {
                    kind:  EntityKind::Hashtag,
                    range: (0, 8),
                },
                Entity {
                    kind:  EntityKind::Url,
                    range: (9, 28),
                },
                Entity {
                    kind:  EntityKind::Mention("mention".to_string(), None),
                    range: (29, 37),
                },
            ]
        );
    }

    // #[test]
    // fn all_tlds_parse() {
    //     let tests = yaml_rust::YamlLoader::load_from_str(TLDS_YAML).unwrap();
    //     let tests = tests.first().unwrap();
    //     let ref tests = tests["tests"];
    //     for (suite, test_cases) in tests.as_hash().expect("could not load tests document") {
    //         let suite = suite.as_str().expect("suite could not be loaded");
    //         for test in test_cases.as_vec().expect("suite could not be loaded") {
    //             let description = test["description"]
    //                 .as_str()
    //                 .expect("test was missing 'description'");
    //             let text = test["text"].as_str().expect("test was missing 'text'");
    //             let expected = test["expected"]
    //                 .as_vec()
    //                 .expect("test was missing 'expected'")
    //                 .iter()
    //                 .map(|s| s.as_str().expect("non-string found in 'expected'"))
    //                 .collect::<HashSet<_>>();

    //             let actual = extract_urls(text)
    //                 .into_iter()
    //                 .map(|e| e.substr(text))
    //                 .collect::<HashSet<_>>();

    //             assert_eq!(
    //                 actual, expected,
    //                 "test {}/\"{}\" failed on text \"{}\"",
    //                 suite, description, text
    //             );
    //         }
    //     }
    // }
}
