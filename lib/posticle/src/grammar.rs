#[allow(unused_imports)]
use pest::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct Grammar;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hashtag() {
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
    fn parses_mention() {
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
    fn parses_url() {
        let valid_urls = vec![
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
        let invalid_urls = vec![
            "http://example.com/\">",
            "http://example.com/\">xyz ",
            "http://example.com/#anchor ",
            "https://example.com.",
            "https://example.com?",
        ];

        for url in valid_urls {
            assert_eq!(
                url,
                Grammar::parse(Rule::url, url)
                    .unwrap_or_else(|e| panic!("{}", e))
                    .as_str()
            );
        }

        for url in invalid_urls {
            assert_ne!(
                url,
                Grammar::parse(Rule::url, url)
                    .unwrap_or_else(|e| panic!("{}", e))
                    .as_str()
            );
        }
    }
}
