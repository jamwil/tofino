use std::{error::Error, fmt::Display, str::FromStr};

pub trait Request {
    fn get(&self) -> Result<impl Response, Box<dyn Error>>;
}

pub trait Response: FromStr + Display {}

pub mod http;
pub mod https;
