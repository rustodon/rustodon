use ammonia::Builder;

pub fn summary(summary: String) -> String {
    let allowed_tags = hashset!["p", "br"];
    Builder::default()
        .tags(allowed_tags)
        .link_rel(Some("noopener nofollow"))
        .clean(&summary)
        .to_string()
}
