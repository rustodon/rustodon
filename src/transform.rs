use ammonia::Builder;
use failure::Error;
use posticle::{self, EntityKind};

use db;
use db::models::Account;

pub fn bio(text: &str, db_conn: &db::Connection) -> Result<String, Error> {
    // prestrip tags (i don't like this either, but I Want To Be Done Okay)
    let text = Builder::default()
        .tags(hashset![])
        .link_rel(Some("noopener nofollow"))
        .clean(text)
        .to_string();

    let mut html = String::new();
    let mut cursor = 0;

    let entities = posticle::entities(&text);

    for entity in entities {
        html.push_str(&text[cursor..entity.range.0]);
        let entity_text = entity.substr(&text);
        let replacement = match entity.kind {
            EntityKind::Url => format!("<a href=\"{url}\">{url}</a>", url = entity_text),
            EntityKind::Hashtag => format!("<a href=\"#\">{hashtag}</a>", hashtag = entity_text),
            EntityKind::Mention(user, domain) => {
                if let Some(account) = Account::fetch_by_username_domain(db_conn, user, domain)? {
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
        cursor = entity.range.1;
    }
    html.push_str(&text[cursor..]);

    Ok(Builder::default()
        .tags(hashset!["a", "p", "br"])
        .link_rel(Some("noopener nofollow"))
        .clean(&html)
        .to_string())
}
