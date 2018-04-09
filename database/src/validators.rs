use regex::Regex;

lazy_static! {
    /// During registrations, usernames must be matched by this regex to be considered valid.
    pub static ref VALID_USERNAME_RE: Regex = Regex::new(r"^[[:alnum:]_]+$").unwrap();
}
