#![feature(nll)]
extern crate regex;
#[macro_use]
extern crate lazy_static;

#[macro_use]
#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

mod regexes;

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
}

/// Given `text`, extract all [Entities](Entity)
pub fn entities(text: &str) -> Vec<Entity> {
    if text.is_empty() {
        return Vec::new();
    }

    let mut results = extract_urls(text);
    results.extend(extract_hashtags(text, &results));
    results.extend(extract_mentions(text, &results));

    results.sort_by(|a, b| a.range.cmp(&b.range));
    results
}

/// Given `text`, extract all [URL](EntityKind::Url) entities.
pub fn extract_urls(text: &str) -> Vec<Entity> {
    regexes::RE_URL
        .find_iter(text)
        .map(|mat| Entity {
            kind:  EntityKind::Url,
            range: (mat.start(), mat.end()),
        }).collect()
}

/// Given `text` and some `existing` entities, extract all [Hashtag](EntityKind::Hashtag) entities
/// which do not overlap with the `existing` ones.
pub fn extract_hashtags(text: &str, existing: &[Entity]) -> Vec<Entity> {
    regexes::RE_HASHTAG
        .find_iter(text)
        .map(|mat| Entity {
            kind:  EntityKind::Hashtag,
            range: (mat.start(), mat.end()),
        }).filter(|en| {
            existing
                .iter()
                .all(|existing_en| !en.overlaps_with(existing_en))
        }).collect()
}

/// Given `text` and some `existing` entities, extract all [Mention](EntityKind::Mention) entities
/// which do not overlap with the `existing` ones.
pub fn extract_mentions(text: &str, existing: &[Entity]) -> Vec<Entity> {
    regexes::RE_MENTION
        .captures_iter(text)
        .map(|capt| {
            let whole = capt.get(0).unwrap();
            let user = capt[1].to_string();
            let domain = capt.get(2).map(|s| s.as_str().to_string());
            Entity {
                kind:  EntityKind::Mention(user, domain),
                range: (whole.start(), whole.end()),
            }
        }).filter(|en| {
            existing
                .iter()
                .all(|existing_en| !en.overlaps_with(existing_en))
        }).collect()
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
