use ammonia::Builder;

pub fn summary<S>(summary: S) -> String
where
    S: AsRef<str>,
{
    let allowed_tags = hashset!["p", "br"];
    Builder::default()
        .tags(allowed_tags)
        .link_rel(Some("noopener nofollow"))
        .clean(summary.as_ref())
        .to_string()
}
