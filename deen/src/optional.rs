use std::{fmt, io, marker::PhantomData};

use core::convert::TryFrom;

use crate::{Deen, invalid_data_error};

pub struct Optional<I> {
    p: PhantomData<I>,
}

impl<I> Optional<I> {
    pub fn wrap<T: Deen>(encoder: T) -> OptionalWithEncoder<I, T> {
        OptionalWithEncoder {
            encoder,
            p: PhantomData,
        }
    }
}

pub struct OptionalWithEncoder<I, T> {
    encoder: T,
    p: PhantomData<I>,
}

impl<I, T> OptionalWithEncoder<I, T> {
    pub fn decode_when<F>(self, pred: F) -> OptionalImpl<I, T, F> {
        OptionalImpl {
            encoder: self.encoder,
            pred,
            p: PhantomData,
        }
    }
}

pub struct OptionalImpl<I, T, F> {
    encoder: T,
    pred: F,
    p: PhantomData<I>,
}

impl<I, T, F> Deen for OptionalImpl<I, T, F>
where
    T: Deen,
    <T as Deen>::Item: TryFrom<I> + Clone,
    <<T as Deen>::Item as TryFrom<I>>::Error: fmt::Display,
    I: TryFrom<<T as Deen>::Item> + Clone,
    <I as TryFrom<<T as Deen>::Item>>::Error: fmt::Display,
    F: Fn() -> bool,
{
    type Item = Option<I>;

    fn encode(&self, value: &Self::Item, buf: impl io::Write) -> io::Result<()> {
        if let Some(v) = &value {
            let t = <T as Deen>::Item::try_from(v.clone()).map_err(invalid_data_error)?;
            self.encoder.encode(&t, buf)?;
        }

        Ok(())
    }

    fn decode(&self, buf: impl io::Read) -> io::Result<Self::Item> {
        let r = if (self.pred)() {
            let t = self.encoder.decode(buf)?;
            let v = I::try_from(t.clone()).map_err(invalid_data_error)?;
            Some(v)
        } else {
            None
        };

        Ok(r)
    }
}
