use waitfree_sync::triple_buffer::{Reader, Writer};

pub trait ReadPrimitive<T> {
    fn read(&mut self) -> Option<T>
    where
        T: Clone;
}
pub trait WritePrimitive<T, E> {
    fn write(&mut self, data: T) -> Result<(), E>;
}

impl<T> ReadPrimitive<T> for Reader<T> {
    #[inline]
    fn read(&mut self) -> Option<T>
    where
        T: Clone,
    {
        self.read()
    }
}

impl<T> WritePrimitive<T, ()> for Writer<T> {
    fn write(&mut self, data: T) -> Result<(), ()> {
        self.write(data);
        Ok(())
    }
}
