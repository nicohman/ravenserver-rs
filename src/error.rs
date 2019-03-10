use failure::Error;
/// A custom result type
pub type Result<T> = std::result::Result<T, failure::Error>;

