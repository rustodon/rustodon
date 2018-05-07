extern crate regex;
#[macro_use]
extern crate lazy_static;

#[macro_use]
#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

mod regexes;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash)]
/// Tags a given [Entity]'s semantic kind.
pub enum EntityKind {
    /// A URL.
    Url,
    /// A hashtag.
    Hashtag,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash)]
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
}

/// Given `text`, extract all [Entities](Entity)
pub fn entities(text: &str) -> Vec<Entity> {
    if text.is_empty() {
        return Vec::new();
    }

    let mut results = url_entities(text);

    results.sort();
    results
}

/// Given `text`, extract all [URL](EntityKind::Url) entities.
pub fn url_entities(text: &str) -> Vec<Entity> {
    regexes::RE_URL.find_iter(text).map(|mat| {
        Entity {
            kind: EntityKind::Url,
            range: (mat.start(), mat.end()),
        }
    }).collect()
}

#[cfg(test)]
mod test {
    extern crate yaml_rust;
    use super::*;
    use std::collections::HashSet;

    const TLDS_YAML: &'static str = include_str!("../vendor/test/tlds.yml");

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

                let actual = url_entities(text)
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
