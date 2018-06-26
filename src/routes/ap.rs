use rocket::Route;

use activitypub::{ActivityGuard, ActivityStreams, AsActivityPub};
use db;
use db::models::{Account, Status};
use error::Perhaps;

pub fn routes() -> Vec<Route> {
    routes![ap_user_object, ap_status_object,]
}

/// Returns a user as an ActivityPub object.
#[get("/users/<username>", rank = 2)]
pub fn ap_user_object(
    username: String,
    _ag: ActivityGuard,
    db_conn: db::Connection,
) -> Perhaps<ActivityStreams> {
    let account = try_resopt!(Account::fetch_local_by_username(&db_conn, username));

    Ok(Some(account.as_activitypub(&db_conn)?))
}

/// Returns a user status as an ActivityPub object.
#[get("/users/<username>/statuses/<status_id>", rank = 2)]
pub fn ap_status_object(
    username: String,
    status_id: u64,
    _ag: ActivityGuard,
    db_conn: db::Connection,
) -> Perhaps<ActivityStreams> {
    let account = try_resopt!(Account::fetch_local_by_username(&db_conn, username));
    let status = try_resopt!(Status::by_account_and_id(
        &db_conn,
        account.id,
        status_id as i64
    ));

    Ok(Some(status.as_activitypub(&db_conn)?))
}
