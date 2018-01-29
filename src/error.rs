/// A type which could be nonexistent, existent, or errored;
/// used for routes which might return {something, a 404, a 500}.
pub type Perhaps<T> = Result<Option<T>, ::failure::Error>;
