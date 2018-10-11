#[allow(unused_imports)]
use pest::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct Grammar;

#[cfg(test)]
mod tests {
    extern crate yaml_rust;
    use super::*;

    const TLDS_YAML: &'static str = include_str!("../vendor/test/tlds.yml");

    #[test]
    fn parses_emoticons() {
        let english = ":rustodon:";
        let japanese = ":文字化け:";

        assert_eq!(
            english,
            Grammar::parse(Rule::emoticon, english).unwrap().as_str()
        );
        assert_eq!(
            japanese,
            Grammar::parse(Rule::emoticon, japanese).unwrap().as_str()
        );
    }

    #[test]
    fn parses_hashtags() {
        let english = "#rustodon";
        let japanese = "#文字化け";

        assert_eq!(
            english,
            Grammar::parse(Rule::hashtag, english).unwrap().as_str()
        );
        assert_eq!(
            japanese,
            Grammar::parse(Rule::hashtag, japanese).unwrap().as_str()
        );
    }

    #[test]
    fn parses_mentions() {
        let valid_mentions = vec!["@noot", "@noot@noot.social", "@no_ot3@noot.social"];
        let invalid_mentions = vec!["@noot@@noot.social", "@@noot@noot.social"];

        for mention in valid_mentions {
            assert_eq!(
                mention,
                Grammar::parse(Rule::mention, mention)
                    .unwrap_or_else(|e| panic!("{}", e))
                    .as_str()
            );
        }

        for mention in invalid_mentions {
            assert!(Grammar::parse(Rule::mention, mention).is_err());
        }
    }

    #[test]
    fn parses_links() {
        let valid_links = vec![
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
            "http://www.-domain4352.com/",
            "http://www.domain4352-.com/",
            "http://☃-.net/",
            "http://%e2%98%83.net/",
        ];
        let invalid_links = vec![
            "http://example.com/\"> ",
            "http://example.com/\">xyz ",
            "http://example.com/#anchor ",
            "https://example.com.",
            "https://example.com?",
        ];

        for link in valid_links {
            assert_eq!(
                link,
                Grammar::parse(Rule::link, link)
                    .unwrap_or_else(|e| panic!("{}", e))
                    .as_str()
            );
        }

        for link in invalid_links {
            assert_ne!(
                link,
                Grammar::parse(Rule::link, link)
                    .unwrap_or_else(|e| panic!("{}", e))
                    .as_str()
            );
        }
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
                    .first()
                    .expect("test was missing items for 'expected'")
                    .as_str()
                    .expect("non-string found in 'expected'");
                let result = Grammar::parse(Rule::link, text).unwrap().as_str();

                assert_eq!(
                    result, expected,
                    "test {}/\"{}\" failed on text \"{}\"",
                    suite, description, text
                );
            }
        }
    }
}
