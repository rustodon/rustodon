use diesel::prelude::*;
use rocket::Route;
use ::db;
use db::models::User;

pub fn routes() -> Vec<Route> {
    routes![index, user_page]
}

#[get("/users/<username>", format="text/html")]
pub fn user_page(username: String, db_conn: db::Connection) -> Option<String> {
    let user = try_opt!(User::by_username(&db_conn, username));

    Some(format!("{:?}", user))
}

#[get("/")]
pub fn index(db_conn: db::Connection) -> String {
    use db::schema::users::dsl::*;

    let found_users = users.load::<User>(&*db_conn)
        .expect("error loading users");
    format!("users: {:?}", found_users)
}
