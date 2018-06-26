use itertools::Itertools;
use rocket::http::ContentType;
use rocket::response::Content;
use rocket::Route;
use rocket_contrib::Json;
use failure::Error;

use db;
use db::models::{Account, Status, User};
use error::Perhaps;
use {BASE_URL, DOMAIN, GIT_REV};

pub fn routes() -> Vec<Route> {
    routes![webfinger_get_resource, webfinger_host_meta, webfinger_nodeinfo, nodeinfo]
}

/// A type representing the parameters of a WebFinger query.
#[derive(FromForm, Debug)]
pub struct WFQuery {
    resource: String,
}

/// Returns JRD replies to `acct:` webfinger queries; required for Mastodon to resolve our accounts.
#[get("/.well-known/webfinger?<query>")]
pub fn webfinger_get_resource(query: WFQuery, db_conn: db::Connection) -> Perhaps<Content<Json>> {
    // TODO: don't unwrap
    let (_, addr) = query
        .resource
        .split_at(query.resource.rfind("acct:").unwrap() + "acct:".len());
    let (username, domain) = addr.split('@').collect_tuple().unwrap();

    // If the webfinger address had a different domain, 404 out.
    if domain != DOMAIN.as_str() {
        return Ok(None);
    }

    let account = try_resopt!(Account::fetch_local_by_username(&db_conn, username));

    let wf_doc = json!({
        "aliases": [account.get_uri()],
        "links": [
            {
                "href": account.get_uri(),
                "rel": "http://webfinger.net/rel/profile-page",
                "type": "text/html",
            },
            {
                "href": account.get_uri(),
                "rel": "self",
                "type": "application/activity+json",
            },
        ],
        "subject": query.resource,
    });

    let wf_content = ContentType::new("application", "jrd+json");

    Ok(Some(Content(wf_content, Json(wf_doc))))
}

/// Returns metadata about well-known routes as XRD; necessary to be Webfinger-compliant.
#[get("/.well-known/host-meta")]
pub fn webfinger_host_meta() -> Content<String> {
    let xrd_xml = ContentType::new("application", "xrd+xml");

    Content(xrd_xml, format!(r#"<?xml version="1.0"?>
<XRD xmlns="http://docs.oasis-open.org/ns/xri/xrd-1.0">
  <Link rel="lrdd" type="application/xrd+xml" template="{base}/.well-known/webfinger?resource={{uri}}"/>
</XRD>"#, base=BASE_URL.as_str()))
}

/// Returns a JRD document referencing (via `Link`s) the NodeInfo documents we support.
#[get("/.well-known/nodeinfo")]
pub fn webfinger_nodeinfo() -> Content<Json> {
    let jrd_ctype = ContentType::new("application", "jrd+json");
    let doc = json!({
        "links": [
            {
                "rel": "http://nodeinfo.diaspora.software/ns/schema/2.0",
                "href": format!("{base}/nodeinfo/2.0", base=BASE_URL.as_str()),
            }
        ]
    });
    Content(jrd_ctype, Json(doc))
}

#[get("/nodeinfo/2.0", format = "application/json")]
pub fn nodeinfo(db_conn: db::Connection) -> Result<Content<Json>, Error> {
    let ctype = ContentType::with_params(
        "application",
        "json",
        (
            "profile",
            "http://nodeinfo.diaspora.software/ns/schema/2.0#,",
        ),
    );
    let doc = json!({
        "version": 2.0,
        "software": {
            "name": "rustodon",
            "version": GIT_REV,
        },
        "protocols": ["activitypub"],
        "services": {"inbound": [], "outbound": []},
        "openRegistrations": true, // TODO: hahaha
        "usage": { // TODO: cache?
            "users": {"total": User::count(&db_conn)?},
            "localPosts": Status::count_local(&db_conn)?,
        }
    });

    Ok(Content(ctype, Json(doc)))
}
