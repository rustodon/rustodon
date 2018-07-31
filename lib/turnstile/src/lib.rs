pub trait Job<T> {
    fn run(&self) -> T;
}
