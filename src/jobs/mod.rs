//! Asynchronous jobs.

use bincode;
use turnstile::Job;

#[derive(Serialize, Deserialize, Debug)]
enum Jobs {
    TestJob(TestJob),
}

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn foo() {
        let j = TestJob { some_data: 413 };
        println!("{:?}", j);
        let bytes = bincode::serialize(&Jobs::TestJob(j)).unwrap();
        println!("{:?}", bytes);

        let j_: Jobs = bincode::deserialize(&bytes).unwrap();
        println!("{:?}", j_);
    }
}
