use std::fmt::{Debug, Display};

pub(crate) fn log_error<E: Display + Debug>(error: E) {
    println!("{:?}", error);
    println!("{}", error);
}
