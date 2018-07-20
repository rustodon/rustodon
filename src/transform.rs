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
        html.push_str(&escape_html(&text[cursor..entity.range.0])?);
        let entity_text = entity.substr(&text);
        let replacement = match entity.kind {
            EntityKind::Url => format!("<a href=\"{url}\">{url}</a>", url = entity_text),
            EntityKind::Hashtag => format!("<a href=\"#\">{hashtag}</a>", hashtag = entity_text),
            EntityKind::Mention(user, domain) => {
                if let Some(account) = account_lookup(&user, domain.as_ref().map(String::as_str))? {
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
    html.push_str(&escape_html(&text[cursor..])?);

    Ok(Builder::default()
        .tags(hashset!["a", "p", "br"])
        .link_rel(Some("noopener nofollow"))
        .clean(&html)
        .to_string())
}
