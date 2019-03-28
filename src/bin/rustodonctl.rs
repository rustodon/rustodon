use dotenv::dotenv;
use std::env;
use structopt::StructOpt;
use prettytable::{self, Table, Row, Cell, row, cell};

use rustodon::db;

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

    /// Dumps the current jobs table.
    /// Can be used to inspect job statuses, similar to looking at Sidekiq when running Mastodon.
    #[structopt(name = "list-jobs")]
    ListJobs,
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
            use diesel::prelude::*;
            use rustodon::db::models::Account;
            use rustodon::db::schema::{accounts, users};

            let needs_keys = accounts::table
                .inner_join(users::table)
                .filter(accounts::privkey.eq(Vec::new()))
                .select(accounts::all_columns)
                .load::<Account>(&db_conn)?;

            for account in needs_keys {
                print!("generating keypair for user {}... ", account.username);
                let keypair =
                    rustodon::crypto::generate_keypair().expect("couldn't generate a keypair!");

                print!("saving... ");
                account
                    .save_keypair(&db_conn, keypair)
                    .expect("error saving keypair!");

                println!("done!");
            }
        },
        Command::ListJobs => {
            let job_list = {
                use diesel::prelude::*;
                use rustodon::db::models::JobRecord;
                use rustodon::db::schema::jobs::dsl::*;

                jobs
                    .order(id.asc())
                    .load::<JobRecord>(&db_conn)
                    .expect("couldn't load from job queue")
            };

            let mut table = Table::new();
            table.set_format(*prettytable::format::consts::FORMAT_BOX_CHARS);
            table.set_titles(row!["ID", "CREATION TIME", "STATUS", "QUEUE", "KIND", "PARAMS"]);

            for job in job_list {
                table.add_row(row![job.id, job.created_at, job.status, job.queue, job.kind, job.data]);
            }

            table.printstd();
        },
    }

    Ok(())
}
