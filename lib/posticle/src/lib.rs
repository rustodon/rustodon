#![feature(nll)]

#[macro_use]
extern crate nom;
extern crate unic_ucd_category;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

use nom::IResult;

mod parsers;

/// Represents an entity extracted from a given string of text.
///
/// The entity is described by its semantic kind and the `data` it comprises within a string.
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Entity<'a> {
    /// A URL.
    Url(&'a str),
    /// A hashtag.
    Hashtag(&'a str),
    /// A mention; the inner data is `(username, optional domain)`.
    Mention(&'a str, Option<&'a str>),
}

/// Represents a mix of entities and text.
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum SemanticText<'a> {
    /// Normal text.
    Text(&'a str),
    /// An entity.
    Entity(Entity<'a>),
}

/// Given `text`, extract all [Entities](Entity)
fn entities_p<'a>(text: &'a str) -> IResult<&'a str, Vec<SemanticText<'a>>> {
    let stext: IResult<&'a str, Vec<SemanticText<'a>>>;
    {
        use nom::AtEof;
        let mut res = Vec::new();
        let mut input = text.clone();
        let mut text_chars = 0;
        loop {
            let (text_input, input_) = input.split_at(text_chars);
            match alt_complete!(
                input_,
                map!(parsers::valid_url, Entity::Url)
                    | map!(parsers::hashtag, Entity::Hashtag)
                    | map!(parsers::mention, |(u, d)| Entity::Mention(u, d))
            ) {
                Ok((i, o)) => {
                    // loop trip must always consume (otherwise infinite loops)
                    if i == input {
                        if i.at_eof() {
                            stext = Ok((input, res));
                        } else {
                            stext = Err(nom::Err::Error(error_position!(
                                input,
                                nom::ErrorKind::Many0
                            )));
                        }
                        break;
                    }
                    if text_chars > 0 {
                        res.push(SemanticText::Text(text_input));
                        text_chars = 0;
                    }
                    res.push(SemanticText::Entity(o));
                    input = i;
                },
                Err(nom::Err::Error(_)) => {
                    if input_.is_empty() {
                        if !text_input.is_empty() || !res.is_empty() {
                            res.push(SemanticText::Text(text_input));
                        }
                        stext = Ok((text_input, res));
                        break;
                    } else {
                        text_chars += 1;
                    }
                },
                Err(e) => {
                    stext = Err(e);
                    break;
                },
            }
        }
    }
    stext
}

pub fn entities<'a>(text: &'a str) -> Option<Vec<SemanticText<'a>>> {
    entities_p(text).map(|r| r.1).ok()
}

#[cfg(test)]
mod tests {
    extern crate yaml_rust;
    use super::*;
    use std::collections::HashSet;

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
                kind:  EntityKind::Mention("mention", None),
                range: (0, 8),
            }]
        );
        assert_eq!(
            entities("@mention@domain.place"),
            vec![Entity {
                kind:  EntityKind::Mention("mention", Some("domain.place")),
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
                    kind:  EntityKind::Mention("mention", None),
                    range: (29, 37),
                },
            ]
        );
    }

    #[test]
    fn all_tlds_parse() {
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
                let expected = test["expected"]
                    .as_vec()
                    .expect("test was missing 'expected'")
                    .iter()
                    .map(|s| s.as_str().expect("non-string found in 'expected'"))
                    .collect::<HashSet<_>>();

                let actual = extract_urls(text)
                    .into_iter()
                    .map(|e| e.substr(text))
                    .collect::<HashSet<_>>();

                assert_eq!(
                    actual, expected,
                    "test {}/\"{}\" failed on text \"{}\"",
                    suite, description, text
                );
            }
        }
    }
}
