use std::error::Error;

pub (crate) fn log_error<E: Error>(error: E) {
    println!("{:?}", error);
    println!("{}", error);
}