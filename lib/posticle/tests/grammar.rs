#[macro_use]
extern crate pretty_assertions;
extern crate posticle;
extern crate yaml_rust;

use posticle::grammar::*;

#[test]
fn parses_emoticons() {
    let english = ":rustodon:";
    let japanese = ":文字化け:";

    assert_eq!(english, emoticon(english).unwrap().as_str());
    assert_eq!(japanese, emoticon(japanese).unwrap().as_str());
}

#[test]
fn parses_hashtags() {
    let english = "#rustodon";
    let japanese = "#文字化け";

    assert_eq!(english, hashtag(english).unwrap().as_str());
    assert_eq!(japanese, hashtag(japanese).unwrap().as_str());
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
        "https://en.wikipedia.org/wiki/Diaspora_(software)",
    ];
    let invalid_links = vec![
        "http://example.com/\"> ",
        "http://example.com/\">xyz ",
        "http://example.com/#anchor ",
        "https://example.com.",
        "https://example.com?",
    ];

    for valid_link in valid_links {
        assert_eq!(
            valid_link,
            link(valid_link)
                .unwrap_or_else(|e| panic!("{}", e))
                .as_str()
        );
    }

    for invalid_link in invalid_links {
        assert_ne!(
            invalid_link,
            link(invalid_link)
                .unwrap_or_else(|e| panic!("{}", e))
                .as_str()
        );
    }
}

#[test]
fn parse_all_links() {
    let tests = include_str!("data/links.yml");
    let tests = yaml_rust::YamlLoader::load_from_str(tests).unwrap();
    let tests = tests.first().unwrap();
    let ref tests = tests["tests"];

    for test_link in tests.as_vec().expect("error reading tests") {
        let expected = test_link.as_str().expect("expected a string");
        let result = link(expected).unwrap().as_str();

        assert_eq!(
            result, expected,
            "parse_all_urls failed on url \"{}\"",
            expected
        );
    }
}

#[test]
fn parses_all_tlds() {
    let tests = include_str!("data/tlds.yml");
    let tests = yaml_rust::YamlLoader::load_from_str(tests).unwrap();
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
            let result = link(text).unwrap().as_str();

            assert_eq!(
                result, expected,
                "test {}/\"{}\" failed on text \"{}\"",
                suite, description, text
            );
        }
    }
}

#[test]
fn parses_mentions() {
    let valid_mentions = vec!["@noot", "@noot@noot.social", "@no_ot3@noot.social"];
    let invalid_mentions = vec!["@noot@@noot.social", "@@noot@noot.social"];

    for valid_mention in valid_mentions {
        assert_eq!(
            valid_mention,
            mention(valid_mention)
                .unwrap_or_else(|e| panic!("{}", e))
                .as_str()
        );
    }

    for invalid_mention in invalid_mentions {
        assert!(mention(invalid_mention).is_err());
    }
}
