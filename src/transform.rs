use ammonia::Builder;
use failure::Error;
use maud_htmlescape::Escaper;
use posticle::{self, EntityKind};

use db;
use db::models::Account;

fn escape_html(text: impl AsRef<str>) -> String {
    use std::fmt::Write;

    let mut out = String::new();
    Escaper::new(&mut out).write_str(text.as_ref());

    out
}

/// TODO: This should likely not require a db connection, the caller should provide a map of usernames to
/// urls or similar.
pub fn bio(text: &str, db_conn: &db::Connection) -> Result<String, Error> {
    let mut html = String::new();
    let mut cursor = 0;

    let entities = posticle::entities(&text);

    for entity in entities {
        html.push_str(&escape_html(&text[cursor..entity.range.0]));
        let entity_text = entity.substr(&text);
        let replacement = match entity.kind {
            EntityKind::Url => format!("<a href=\"{url}\">{url}</a>", url = entity_text),
            EntityKind::Hashtag => format!("<a href=\"#\">{hashtag}</a>", hashtag = entity_text),
            EntityKind::Mention(user, domain) => {
                if let Some(account) = Account::fetch_by_username_domain(db_conn, user, domain)? {
                    format!(
                        "<a href=\"{url}\">{mention}</a>",
                        url = account.profile_path(),
                        mention = entity_text
                    )
                } else {
                    entity_text.into()
                }
            },
        };
        html.push_str(&replacement);
        cursor = entity.range.1;
    }
    html.push_str(&escape_html(&text[cursor..]));

    Ok(Builder::default()
        .tags(hashset!["a", "p", "br"])
        .link_rel(Some("noopener nofollow"))
        .clean(&html)
        .to_string())
}
