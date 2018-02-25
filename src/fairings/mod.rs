use db::{Connection, Pool};
use rocket::Rocket;
use rocket::fairing::{Fairing, Info, Kind};
use db::snowflake::init;

pub struct SnowflakeInitFairing;

impl Fairing for SnowflakeInitFairing {
    fn info(&self) -> Info {
        Info {
            name: "Snowflake ID generator initializer",
            kind: Kind::Attach,
        }
    }

    fn on_attach(&self, rocket: Rocket) -> Result<Rocket, Rocket> {
        let pool = rocket.state::<Pool>();

        match pool {
            Some(p) => {
                match p.get() {
                    Ok(c) => match init(Connection(c)) {
                        Ok(_) => Ok(rocket),
                        Err(s) => {
                            println!("failed to initialize ID generator: {}", s);
                            Err(rocket)
                        },
                    },
                    Err(_) => {
                        println!("failed to initialize ID generator: could not acquire database connection");
                        Err(rocket)
                    },
                }
            },
            None => {
                println!("failed to initialize ID generator: no connection pool available");
                Err(rocket)
            },
        }
    }
}
