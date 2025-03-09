use alloc::string::String;

#[derive(Debug)]
pub enum Error {
    Network(String),
    UnexpectedInput(String),
    InvalidUI(String),
    Other(String),
}
