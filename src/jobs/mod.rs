//! Asynchronous jobs.

use serde_json;
use turnstile::Job;

#[derive(Serialize, Deserialize, Debug)]
struct TestJob {
    some_data: u32,
}

impl Job<()> for TestJob {
    fn run(&self) -> Result<(), ()> {
        println!("self.some_data: {:?}", self.some_data);

        Ok(())
    }
}
