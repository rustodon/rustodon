extern crate rocket;
#[macro_use] extern crate diesel;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate dotenv;
use dotenv::dotenv;

fn main() {
    dotenv().ok();

    rocket::ignite()
        .launch();
}
