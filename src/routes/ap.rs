use itertools::Itertools;
use rocket::{Route, Outcome};
use rocket::response::Content;
use rocket::request::{self, Request, FromRequest};
use rocket::http::{Accept, MediaType, ContentType};
use rocket_contrib::Json;

use db;
use db::models::Account;
use activitypub::{ActivityStreams, AsActivityPub};
use ::BASE_URL;


pub fn routes() -> Vec<Route> {
    routes![ap_user_object, webfinger_get_resource, webfinger_host_meta]
}

pub struct ActivityGuard();
impl<'a, 'r> FromRequest<'a, 'r> for ActivityGuard {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<ActivityGuard, ()> {
        if request.accept().map(is_ap).unwrap_or(false) {
            Outcome::Success(ActivityGuard())
        } else {
            Outcome::Forward(())
        }
    }
}

pub fn is_ap(accept: &Accept) -> bool {
    let media_type = accept.preferred().media_type();

    // TODO: clean this up/make these const, if MediaType::new ever becomes a const fn
    let ap_json = MediaType::new("application", "activity+json");
    let ap_json_ld = MediaType::with_params("application", "ld+json", ("profile", "https://www.w3.org/ns/activitystreams"));

    media_type.exact_eq(&ap_json) || media_type.exact_eq(&ap_json_ld)
}

#[get("/users/<username>", rank=2)]
pub fn ap_user_object(username: String, _ag: ActivityGuard, db_conn: db::Connection) -> Option<ActivityStreams> {
    let account = try_opt!(Account::fetch_local_by_username(&db_conn, username));

    Some(account.as_activitypub())
}

#[derive(FromForm, Debug)]
pub struct WFQuery {
    resource: String,
}

#[get("/.well-known/webfinger?<query>")]
pub fn webfinger_get_resource(query: WFQuery, db_conn: db::Connection) -> Option<Content<Json>> {
    // TODO: don't unwrap
    let (_, addr) = query.resource.split_at(query.resource.rfind("acct:").unwrap() + "acct:".len());
    let (username, _domain) = addr.split('@').collect_tuple().unwrap();

    // TODO: check domain, don't just assume it's local

    let account = try_opt!(Account::fetch_local_by_username(&db_conn, username));

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

    Some(Content(wf_content, Json(wf_doc)))
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn identifies_ap_requests() {
        use std::str::FromStr;

        let accept_json = Accept::from_str("application/activity+json").unwrap();
        let accept_json_ld = Accept::from_str("application/ld+json; profile=\"https://www.w3.org/ns/activitystreams\"").unwrap();

        assert!(is_ap(&accept_json_ld));
        assert!(is_ap(&accept_json));
    }
}
