use dotenv::dotenv;
use std::env;
use structopt::StructOpt;

use db;

#[derive(Debug, StructOpt)]
#[structopt(name = "rustodonctl")]
struct Opt {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Generates missing keys for local users.
    /// Should be run after db migrations complete when upgrading from pre-HTTP-signatures Rustodon versions.
    #[structopt(name = "generate-keys")]
    GenerateKeys,
}

fn main() -> Result<(), Box<std::error::Error>> {
    // load environment variables fron .env
    dotenv().ok();

    let opt = Opt::from_args();

    // extract the database url from the environment and create the db connection pool
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_conn = db::init_connection(db_url).expect("Couldn't establish connection to database!");

    match opt.cmd {
        Command::GenerateKeys => {
            use db::models::Account;
            use db::schema::{accounts, users};
            use diesel::prelude::*;

            let needs_keys = accounts::table
                .inner_join(users::table)
                .select(accounts::all_columns)
                .load::<Account>(&db_conn)?;

            println!("needs_keys = {:#?}", needs_keys);

            for account in needs_keys {
                let keypair =
                    crate::crypto::generate_keypair().expect("couldn't generate a keypair!");
                println!("{:?}", keypair.public);
            }
        },
    }

    Ok(())
}
