use ammonia::{Builder, Url};
use failure::Error;
use posticle::tokens::*;
use posticle::{Posticle, PosticleConfig};

use db::models::Account;
use error::Perhaps;

struct Biography<'l> {
    account_lookup: Box<Fn(&str, Option<&str>) -> Perhaps<Account> + 'l>,
}

impl<'l> PosticleConfig for Biography<'l> {
    fn html_sanitizer(&self) -> Builder {
        let mut sanitizer = Builder::default();

        sanitizer
            .tags(hashset!["a", "br"])
            .link_rel(Some("noopener nofollow"));

        sanitizer
    }

    fn transform_token(&self, token: Token) -> Vec<Token> {
        match token {
            Token::Hashtag(hashtag) => vec![Token::Element(Element(
                "a".to_string(),
                Some(vec![("href".to_string(), "#".to_string())]),
                Some(format!("#{}", hashtag.0)),
            ))],
            Token::Link(link) => {
                let url = Url::parse(&link.0);

                if let Ok(url) = url {
                    match url.scheme() {
                        "http" | "https" => vec![Token::Element(Element(
                            "a".to_string(),
                            Some(vec![("href".to_string(), link.0.clone())]),
                            Some(link.0),
                        ))],
                        _ => vec![Token::Link(link)],
                    }
                } else {
                    vec![Token::Link(link)]
                }
            },
            Token::Mention(mention) => {
                let account_lookup = &self.account_lookup;
                let lookup = account_lookup(&mention.0, mention.1.as_ref().map(String::as_str));

                if let Ok(Some(account)) = lookup {
                    let mut name = format!("@{}", mention.0);

                    if let Some(domain) = &mention.1 {
                        name.push_str(&format!("@{}", domain));
                    }

                    vec![Token::Element(Element(
                        "a".to_string(),
                        Some(vec![("href".to_string(), account.get_uri().to_string())]),
                        Some(name),
                    ))]
                } else {
                    vec![Token::Mention(mention)]
                }
            },
            _ => vec![token],
        }
    }
}

pub fn bio<L>(text: &str, account_lookup: L) -> Result<String, Error>
where
    L: Fn(&str, Option<&str>) -> Perhaps<Account>,
{
    let posticle = Posticle::from(Biography {
        account_lookup: Box::new(account_lookup),
    });

    Ok(posticle.render(&text))
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
