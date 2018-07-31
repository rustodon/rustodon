pub trait Job<E> {
    fn run(&self) -> Result<(), E>;
}
