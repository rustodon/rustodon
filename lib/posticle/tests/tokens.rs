#[macro_use]
extern crate pretty_assertions;
extern crate posticle;

use posticle::tokens::*;
use posticle::Reader;

#[test]
fn extracts_nothing() {
    assert_eq!(
        Reader::from("a string without at signs").to_vec(),
        vec![Token::Text(Text("a string without at signs".to_string()))]
    );
}

#[test]
fn extracts_mentions() {
    assert_eq!(
        Reader::from("@mention").to_vec(),
        vec![Token::Mention(Mention("mention".to_string(), None))]
    );
    assert_eq!(
        Reader::from("@mention@domain.place").to_vec(),
        vec![Token::Mention(Mention(
            "mention".to_string(),
            Some("domain.place".to_string())
        ))]
    );
    assert_eq!(
        Reader::from("@Mention@Domain.Place").to_vec(),
        vec![Token::Mention(Mention(
            "Mention".to_string(),
            Some("Domain.Place".to_string())
        ))]
    );
}

#[test]
fn extracts_mentions_in_punctuation() {
    assert_eq!(
        Reader::from("(@mention)").to_vec(),
        vec![
            Token::Text(Text("(".to_string())),
            Token::Mention(Mention("mention".to_string(), None)),
            Token::Text(Text(")".to_string()))
        ]
    );
}

#[test]
fn ignores_invalid_mentions() {
    let mentions = vec![
        "some text @ yo",
        "@@yuser@domain",
        "@xuser@@domain",
        "@@not",
        "@not@",
        "@not@@not",
        "@not@not@not",
    ];

    for mention in mentions {
        assert_eq!(
            Reader::from(mention).to_vec(),
            vec![Token::Text(Text(mention.to_string()))],
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
            Reader::from(hashtag).to_vec(),
            vec![Token::Hashtag(Hashtag(hashtag[1..].to_string()))],
            "extracts_hashtags failed on {}",
            hashtag
        );
    }
}

#[test]
fn extracts_hashtags_in_punctuation() {
    let hashtags = vec!["#hashtag", "#HASHTAG", "#1000followers", "#文字化け"];

    for hashtag in hashtags {
        assert_eq!(
            Reader::from(format!("({})", hashtag)).to_vec(),
            vec![
                Token::Text(Text("(".to_string())),
                Token::Hashtag(Hashtag(hashtag[1..].to_string())),
                Token::Text(Text(")".to_string()))
            ],
            "extracts_hashtags_in_punctuation failed on {}",
            hashtag
        );
    }
}

#[test]
fn ignores_invalid_hashtags() {
    let hashtags = vec![
        "some text # yo",
        "##not",
        "#not#",
        "#not##not",
        "#not#not#not",
    ];

    for hashtag in hashtags {
        assert_eq!(
            Reader::from(hashtag).to_vec(),
            vec![Token::Text(Text(hashtag.to_string()))],
            "ignores_invalid_hashtags failed on {}",
            hashtag
        );
    }
}

#[test]
fn extracts_links() {
    let links = vec![
        "http://example.com",
        "http://example.com/path/to/resource?search=foo&lang=en",
        "http://example.com/#!/heck",
        "HTTP://www.ExaMPLE.COM/index.html",
        "http://example.com:8080/",
        "http://test_underscore.example.com",
        "http://☃.net/",
        "http://example.com/Русские_слова",
        "http://example.com/الكلمات_العربية",
        "http://sports.yahoo.com/nfl/news;_ylt=Aom0;ylu=XyZ?slug=ap-superbowlnotebook",
        "http://example.com?foo=$bar.;baz?BAZ&c=d-#top/?stories",
        "http://www.youtube.com/watch?v=g8X0nJHrJ9g&list=PLLLYkE3G1HEAUsnZ-vfTeQ_ZO37DhHhOY-",
        "http://www.example.com/",
    ];

    for link in links {
        assert_eq!(
            Reader::from(link).to_vec(),
            vec![Token::Link(Link(link.to_string()))],
            "extracts_links failed on {}",
            link
        );
    }
}

#[test]
fn extracts_links_in_punctuation() {
    let links = vec![
        "http://example.com",
        "http://example.com/path/to/resource?search=foo&lang=en",
        "http://example.com/#!/heck",
        "HTTP://www.ExaMPLE.COM/index.html",
        "http://example.com:8080/",
        "http://test_underscore.example.com",
        "http://☃.net/",
        "http://example.com/Русские_слова",
        "http://example.com/الكلمات_العربية",
        "http://sports.yahoo.com/nfl/news;_ylt=Aom0;ylu=XyZ?slug=ap-superbowlnotebook",
        "http://example.com?foo=$bar.;baz?BAZ&c=d-#top/?stories",
        "http://www.youtube.com/watch?v=g8X0nJHrJ9g&list=PLLLYkE3G1HEAUsnZ-vfTeQ_ZO37DhHhOY-",
        "http://www.example.com/",
    ];

    for link in links {
        assert_eq!(
            Reader::from(format!("({})", link)).to_vec(),
            vec![
                Token::Text(Text("(".to_string())),
                Token::Link(Link(link.to_string())),
                Token::Text(Text(")".to_string()))
            ],
            "extracts_links_in_punctuation failed on {}",
            link
        );
    }
}

#[test]
fn ignores_invalid_links() {
    let links = vec!["x- text http:// yo", "_=_:thing", "nö://thing/else yo"];

    for link in links {
        assert_eq!(
            Reader::from(link).to_vec(),
            vec![Token::Text(Text(link.to_string()))],
            "ignores_invalid_links failed on {}",
            link
        );
    }
}

#[test]
fn extracts_all() {
    assert_eq!(
        Reader::from("text #hashtag https://example.com @mention text").to_vec(),
        vec![
            Token::Text(Text("text ".to_string())),
            Token::Hashtag(Hashtag("hashtag".to_string())),
            Token::Text(Text(" ".to_string())),
            Token::Link(Link("https://example.com".to_string())),
            Token::Text(Text(" ".to_string())),
            Token::Mention(Mention("mention".to_string(), None)),
            Token::Text(Text(" text".to_string())),
        ]
    );
}
