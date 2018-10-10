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
    static ref VALID_HASHTAG_NAME_RE: Regex = Regex::new(r"^[\w_]*[\p{Alphabetic}_·][\w_]*$").unwrap();

    /// Matches all valid characters in a mention username (after the first @).
    static ref VALID_MENTION_USERNAME_RE: Regex = Regex::new(r"^(?i)[a-z0-9_]+([a-z0-9_\.]+[a-z0-9_]+)?$").unwrap();
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
/// The entity is described by its `kind` and the `span` of indices it occupies within the string.
pub struct Entity {
    pub kind: EntityKind,
    pub span: (usize, usize),
}

impl Entity {
    /// Extracts the span described by this Entity from the passed `text`.
    pub fn substr<'a>(&self, text: &'a str) -> &'a str {
        &text[self.span.0..self.span.1]
    }

    /// Returns `true` if this entity overlaps with some `other` entity.
    pub fn overlaps_with(&self, other: &Entity) -> bool {
        self.span.0 <= other.span.1 && other.span.0 <= self.span.1
    }

    fn from_parse_pair(pair: Pair<Rule>) -> Option<Self> {
        let span = (
            pair.as_span().start_pos().pos(),
            pair.as_span().end_pos().pos(),
        );

        match pair.as_rule() {
            Rule::mention => {
                let mut inner = pair.into_inner();
                let username = inner.next().unwrap().as_str();
                let domain = inner.next().as_ref().map(Pair::as_str);

                if validate_mention_username(username)
                    && (domain.map(validate_mention_domain).unwrap_or(true))
                {
                    Some(Entity {
                        kind: EntityKind::Mention(username.to_string(), domain.map(str::to_string)),
                        span,
                    })
                } else {
                    None
                }
            },
            Rule::url => {
                if validator::validate_url(pair.as_str()) {
                    Some(Entity {
                        kind: EntityKind::Url,
                        span,
                    })
                } else {
                    None
                }
            },
            Rule::hashtag => {
                let name = pair.into_inner().next().unwrap().as_str();
                if validate_hashtag_name(name) {
                    Some(Entity {
                        kind: EntityKind::Hashtag,
                        span,
                    })
                } else {
                    None
                }
            },
            _ => None,
        }
    }
}

/// Given `text`, extract all [Entities](Entity)
pub fn entities(text: &str) -> Vec<Entity> {
    // If the parse succeeded, run Entity::from_parse_pair on each pair, dropping those which returned None;
    // collect to a vec of entities. If the parse errored, just return an empty vec.
    Grammar::parse(Rule::post, text)
        .map(|pairs| pairs.filter_map(Entity::from_parse_pair).collect())
        .unwrap_or_default()
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
                kind: EntityKind::Mention("mention".to_string(), None),
                span: (0, 8),
            }]
        );
        assert_eq!(
            entities("@mention@domain.place"),
            vec![Entity {
                kind: EntityKind::Mention("mention".to_string(), Some("domain.place".to_string())),
                span: (0, 21),
            }]
        );
        assert_eq!(
            entities("@Mention@Domain.Place"),
            vec![Entity {
                kind: EntityKind::Mention("Mention".to_string(), Some("Domain.Place".to_string())),
                span: (0, 21),
            }]
        );
    }

    #[test]
    fn ignores_invalid_mentions() {
        let mentions = vec![
            "some text @ yo",
            "@@yuser@domain",
            "@xuser@@domain",
            "@zuser@-domain-.com",
        ];

        for mention in mentions {
            assert_eq!(
                entities(mention),
                vec![],
                "ignores_invalid_mentions failed on {}",
                mention
            );
        }
    }

    #[test]
    fn extracts_hashtags() {
        let hashtags = vec!["#hashtag", "#HASHTAG", "#1000followers", "#文字化け"];

        for hashtag in hashtags {
            assert_eq!(
                entities(hashtag),
                vec![Entity {
                    kind: EntityKind::Hashtag,
                    span: (0, hashtag.len()),
                }],
                "extracts_hashtags failed on {}",
                hashtag
            );
        }
    }

    #[test]
    fn ignores_invalid_hashtags() {
        let hashtags = vec!["some text # yo", "#---bite-my-entire---", "#123"];

        for hashtag in hashtags {
            assert_eq!(
                entities(hashtag),
                vec![],
                "ignores_invalid_hashtags failed on {}",
                hashtag
            );
        }
    }

    #[test]
    fn extracts_urls() {
        let urls = vec![
            "http://example.com",
            "https://example.com/path/to/resource?search=foo&lang=en",
            "http://example.com/#!/heck",
            "HTTPS://www.ExaMPLE.COM/index.html",
            "https://example.com:8080/",
            "http://test_underscore.example.com",
            "http://☃.net/",
            "http://example.com/Русские_слова",
            "http://example.com/الكلمات_العربية",
            "http://sports.yahoo.com/nfl/news;_ylt=Aom0;ylu=XyZ?slug=ap-superbowlnotebook",
            "http://example.com?foo=$bar.;baz?BAZ&c=d-#top/?stories",
            "https://www.youtube.com/watch?v=g8X0nJHrJ9g&list=PLLLYkE3G1HEAUsnZ-vfTeQ_ZO37DhHhOY-",
            "ftp://www.example.com/",
        ];

        for url in urls {
            assert_eq!(
                entities(url),
                vec![Entity {
                    kind: EntityKind::Url,
                    span: (0, url.len()),
                }],
                "extracts_urls failed on {}",
                url
            );
        }
    }

    #[test]
    fn ignores_invalid_urls() {
        let urls = vec![
            "some text http:// yo",
            "some:thing",
            "some://thing/else yo",
            "http://www.-domain4352.com/",
            "http://www.domain4352-.com/",
            "http://☃-.net/",
        ];

        for url in urls {
            assert_eq!(
                entities(url),
                vec![],
                "ignores_invalid_urls failed on {}",
                url
            );
        }
    }

    #[test]
    fn extracts_all() {
        assert_eq!(
            entities("text #hashtag https://example.com @mention text"),
            vec![
                Entity {
                    kind: EntityKind::Hashtag,
                    span: (5, 13),
                },
                Entity {
                    kind: EntityKind::Url,
                    span: (14, 33),
                },
                Entity {
                    kind: EntityKind::Mention("mention".to_string(), None),
                    span: (34, 42),
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
