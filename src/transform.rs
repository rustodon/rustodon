use ammonia::Builder;
use failure::Error;
use maud_htmlescape::Escaper;
use posticle::{self, EntityKind};

use db::models::Account;
use error::Perhaps;

fn escape_html(text: impl AsRef<str>) -> Result<String, Error> {
    use std::fmt::Write;

    let mut out = String::new();
    Escaper::new(&mut out).write_str(text.as_ref())?;
    out = out.replace("\r", "").replace("\n", "<br>");

    Ok(out)
}

pub fn bio<L>(text: &str, account_lookup: L) -> Result<String, Error>
where
    L: Fn(&str, Option<&str>) -> Perhaps<Account>,
{
    let mut html = String::new();
    let mut cursor = 0;

    let entities = posticle::entities(&text);

    for entity in entities {
        html.push_str(&escape_html(&text[cursor..entity.span.0])?);
        let entity_text = entity.substr(&text);
        let replacement = match entity.kind {
            EntityKind::Url => format!("<a href=\"{url}\">{url}</a>", url = entity_text),
            EntityKind::Hashtag => format!("<a href=\"#\">{hashtag}</a>", hashtag = entity_text),
            EntityKind::Mention(user, domain) => {
                if let Some(account) = account_lookup(&user, domain.as_ref().map(String::as_str))? {
                    format!(
                        "<a href=\"{url}\">{mention}</a>",
                        url = account.get_uri(),
                        mention = entity_text
                    )
                } else {
                    entity_text.into()
                }
            },
        };
        html.push_str(&replacement);
        cursor = entity.span.1;
    }
    html.push_str(&escape_html(&text[cursor..])?);

    Ok(Builder::default()
        .tags(hashset!["a", "p", "br"])
        .link_rel(Some("noopener nofollow"))
        .clean(&html)
        .to_string())
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
        assert_eq!(bio("\n", |_, _| Ok(None)).unwrap(), "<br>");
        assert_eq!(bio("\r\n", |_, _| Ok(None)).unwrap(), "<br>");
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
