use std::{convert::Infallible, error::Error};

pub trait Surface {
    type Frame<'a>
    where
        Self: 'a;
    type Error: Error;

    fn create_frame(&mut self) -> Result<Self::Frame<'_>, Self::Error>;
    fn resize(&mut self, size: (u32, u32)) -> Result<(), Self::Error>;
}

impl Surface for () {
    type Frame<'a> = ();
    type Error = Infallible;

    fn create_frame(&mut self) -> Result<Self::Frame<'_>, Self::Error> {
        Ok(())
    }
    fn resize(&mut self, _: (u32, u32)) -> Result<(), Self::Error> {
        Ok(())
    }
}
