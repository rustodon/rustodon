/// An analog of `try!` for `Result<Option<_>, _>`s.
#[macro_export]
macro_rules! try_resopt {
    ($expr:expr) => {
        match $expr {
            std::result::Result::Ok(opt_val) => match opt_val {
                std::option::Option::Some(val) => val,
                std::option::Option::None => {
                    return std::result::Result::Ok(std::option::Option::None);
                },
            },
            std::result::Result::Err(err) => {
                return std::result::Result::Err(std::convert::From::from(err));
            },
        }
    };
}
