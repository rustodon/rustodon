use flaken::Flaken;
use lazy_static::lazy_static;
use std::cell::Cell;
use std::sync::Mutex;

pub struct IdGenerator {
    flaken: Flaken,
}

/// Constructs an IdGenerator, which can be used to provide one or more snowflake IDs
/// for a database transaction.
///
/// Example use:
///
/// ```
/// # use rustodon_database::idgen::id_generator;
/// # struct ModelA { id: i64 }
/// # struct ModelB { id: i64 }
/// let mut id_gen = id_generator();
///
/// let modelA = ModelA { id: id_gen.next() };
///
/// let modelB = ModelB { id: id_gen.next() };
/// ```
pub fn id_generator() -> IdGenerator {
    IdGenerator {
        flaken: Flaken::default().node(node_id()),
    }
}

lazy_static! {
    static ref THREAD_COUNTER: Mutex<u64> = Mutex::new(0);
}

thread_local! {
    static THREAD_ID: Cell<u64> = Cell::new(0);
}

/// Generates a node ID for the IdGenerator.
fn node_id() -> u64 {
    THREAD_ID.with(|f| {
        let mut g = THREAD_COUNTER.lock().unwrap();
        *g += 1;
        f.set(*g);
        f.get()
    })
}

impl IdGenerator {
    pub fn next(&mut self) -> i64 {
        self.flaken.next() as i64
    }
}
