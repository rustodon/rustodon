use ammonia::{Builder, Url};
use failure::Error;
use posticle::tokens::*;
use posticle::{Reader, Writer};
use regex::Regex;

use db::models::Account;
use error::Perhaps;

lazy_static! {
    /// Matches all valid characters in a hashtag name (after the first #).
    static ref VALID_HASHTAG_NAME_RE: Regex = Regex::new(r"^[\w_]*[\p{Alphabetic}_·][\w_]*$").unwrap();

    /// Matches all valid characters in a mention username (after the first @).
    static ref VALID_MENTION_USERNAME_RE: Regex = Regex::new(r"^(?i)[a-z0-9_]+([a-z0-9_\.]+[a-z0-9_]+)?$").unwrap();
}

pub fn bio<L>(text: &str, account_lookup: L) -> Result<String, Error>
where
    L: Fn(&str, Option<&str>) -> Perhaps<Account>,
{
    let transformer = |token| match token {
        Token::Hashtag(hashtag) => {
            if VALID_HASHTAG_NAME_RE.is_match(&hashtag.name) {
                Token::Element(Element {
                    name: "a".to_string(),
                    attributes: vec![("href".to_string(), "#".to_string())],
                    children: vec![Token::Text(Text {
                        text: format!("#{}", hashtag.name),
                    })],
                })
            } else {
                Token::Hashtag(hashtag)
            }
        },
        Token::Link(link) => {
            let url = Url::parse(&link.url);

            if let Ok(url) = url {
                match url.scheme() {
                    "http" | "https" => Token::Element(Element {
                        name: "a".to_string(),
                        attributes: vec![("href".to_string(), link.url.clone())],
                        children: vec![Token::Text(Text { text: link.url })],
                    }),
                    _ => Token::Link(link),
                }
            } else {
                Token::Link(link)
            }
        },
        Token::Mention(mention) => {
            if VALID_MENTION_USERNAME_RE.is_match(&mention.username) {
                let lookup = account_lookup(
                    &mention.username,
                    mention.domain.as_ref().map(String::as_str),
                );

                if let Ok(Some(account)) = lookup {
                    let mut name = format!("@{}", mention.username);

                    if let Some(domain) = &mention.domain {
                        name.push_str(&format!("@{}", domain));
                    }

                    Token::Element(Element {
                        name: "a".to_string(),
                        attributes: vec![("href".to_string(), account.get_uri().to_string())],
                        children: vec![Token::Text(Text { text: name })],
                    })
                } else {
                    Token::Mention(mention)
                }
            } else {
                Token::Mention(mention)
            }
        },
        _ => token,
    };

    let mut html_sanitizer = Builder::default();

    html_sanitizer
        .tags(hashset!["br", "a"])
        .link_rel(Some("noopener nofollow"));

    let reader = Reader::new()
        .with_transformer(Box::new(transformer))
        .with_str(text)
        .finish();
    let html = Writer::new()
        .with_html_sanitizer(html_sanitizer)
        .with_reader(reader)
        .finish();

    Ok(format!("<p>{}</p>", html.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passes_through_text() {
        assert_eq!(bio("foo", |_, _| Ok(None)).unwrap(), "foo");
        assert_eq!(bio("foo:bar", |_, _| Ok(None)).unwrap(), "foo:bar");
    }

    #[test]
    fn escapes_html_characters() {
        assert_eq!(bio("<>&", |_, _| Ok(None)).unwrap(), "&lt;&gt;&amp;");
        assert_eq!(
            bio("<a></a>", |_, _| Ok(None)).unwrap(),
            "&lt;a&gt;&lt;/a&gt;"
        );
    }

    #[test]
    fn converts_newlines_to_br() {
        assert_eq!(bio("\n", |_, _| Ok(None)).unwrap(), "\n<br>");
        assert_eq!(bio("\r\n", |_, _| Ok(None)).unwrap(), "\n<br>");
    }

    #[test]
    fn converts_links_to_a_tags() {
        assert_eq!(
            bio("https://example.com", |_, _| Ok(None)).unwrap(),
            "<a href=\"https://example.com\" rel=\"noopener nofollow\">https://example.com</a>"
        );
        assert_eq!(
            bio("http://example.com", |_, _| Ok(None)).unwrap(),
            "<a href=\"http://example.com\" rel=\"noopener nofollow\">http://example.com</a>"
        );
        assert_eq!(
            bio("http://‽.com/∰/", |_, _| Ok(None)).unwrap(),
            "<a href=\"http://‽.com/∰/\" rel=\"noopener nofollow\">http://‽.com/∰/</a>"
        );
    }

    #[test]
    fn converts_hashtags_to_links() {
        // TODO: we don't have hashtags atm, so we just fake-link them!
        assert_eq!(
            bio("#hashtag", |_, _| Ok(None)).unwrap(),
            "<a href=\"#\" rel=\"noopener nofollow\">#hashtag</a>"
        );
    }

    #[test]
    fn converts_mentions_to_links() {
        use std::env;
        env::set_var("DOMAIN", "localhost"); // TODO: this is bad and should go away, _somehow_.

        fn acct_lookup(user: &str, domain: Option<&str>) -> Perhaps<Account> {
            Ok(match (user, domain) {
                ("localfoo", None) => Some(Account {
                    id: 0,
                    uri: None,
                    domain: Some("".to_string()),
                    username: "localfoo".to_string(),
                    display_name: None,
                    summary: None,
                }),
                ("remotefoo", Some("remote.example")) => Some(Account {
                    id: 1,
                    uri: Some("https://remote.example/remotefoo".to_string()),
                    domain: Some("remote.example".to_string()),
                    username: "remotefoo".to_string(),
                    display_name: None,
                    summary: None,
                }),
                _ => None,
            })
        }

        assert_eq!(
            bio("@localfoo", acct_lookup).unwrap(),
            "<a href=\"https://localhost/users/localfoo\" rel=\"noopener nofollow\">@localfoo</a>"
        );
        assert_eq!(
            bio("@remotefoo@remote.example", acct_lookup).unwrap(),
            "<a href=\"https://remote.example/remotefoo\" rel=\"noopener nofollow\">@remotefoo@remote.example</a>"
        );
        assert_eq!(bio("@invalid", acct_lookup).unwrap(), "@invalid");
    }
}
