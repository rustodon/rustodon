use rocket::Route;
use rocket_contrib::Template;
use ::db;
use db::models::Account;

pub fn routes() -> Vec<Route> {
    routes![index, user_page]
}

#[get("/users/<username>", format="text/html")]
pub fn user_page(username: String, db_conn: db::Connection) -> Option<Template> {
    let account = try_opt!(Account::fetch_local_by_username(&db_conn, username));

    // We can use a cute hack to remove the need to explicitly write out a context struct,
    // by using the serde_json helper to construct a `Serialize`-able struct on the fly.
    let context = json!({
        "fq_username": account.fully_qualified_username(),
        "display_name": account.display_name.as_ref().unwrap_or(&account.username),
        "uri": &account.get_uri(),
        "bio": account.summary.as_ref().map(String::as_str).unwrap_or("<p></p>"),
    });

    Some(Template::render("user_profile", context))
}

#[get("/")]
pub fn index(db_conn: db::Connection) -> Template {
    Template::render("index", json!({}))
}
