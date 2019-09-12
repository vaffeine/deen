mod integers;
mod optional;

pub use integers::*;
pub use optional::*;

use std::{fmt, io};

pub trait Deen {
    type Item;

    fn encode(&self, value: &Self::Item, buf: impl io::Write) -> io::Result<()>;
    fn decode(&self, buf: impl io::Read) -> io::Result<Self::Item>;
}

pub trait Value {
    fn encode_value(&self, buf: impl io::Write) -> io::Result<()>;
    fn compare(&self, buf: impl io::Read) -> io::Result<()>;
}

pub struct Tag<T: Deen> {
    pub deener: T,
    pub value: <T as Deen>::Item,
}

impl<T> Tag<T>
where
    T: Deen,
    <T as Deen>::Item: PartialEq,
{
    pub fn new(deener: T, value: <T as Deen>::Item) -> Tag<T> {
        Self { deener, value }
    }
}

impl<T> Value for Tag<T>
where
    T: Deen,
    <T as Deen>::Item: PartialEq + fmt::Debug,
{
    fn encode_value(&self, buf: impl io::Write) -> io::Result<()> {
        self.deener.encode(&self.value, buf)
    }

    fn compare(&self, buf: impl io::Read) -> io::Result<()> {
        let other = self.deener.decode(buf)?;

        if other == self.value {
            Ok(())
        } else {
            Err(invalid_data_error(format!(
                "unexpected tag - expected: {:?}, found: {:?}",
                self.value, other
            )))
        }
    }
}

pub struct Any<T: Deen> {
    pub deener: T,
}

impl<T> Any<T>
where
    T: Deen,
{
    pub fn new(deener: T) -> Any<T> {
        Self { deener }
    }
}

impl<T> Value for Any<T>
where
    T: Deen,
    <T as Deen>::Item: Default,
{
    fn encode_value(&self, buf: impl io::Write) -> io::Result<()> {
        self.deener.encode(&<T as Deen>::Item::default(), buf)
    }

    fn compare(&self, buf: impl io::Read) -> io::Result<()> {
        self.deener.decode(buf)?;

        Ok(())
    }
}

fn invalid_data_error<D: fmt::Display>(d: D) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, d.to_string())
}
