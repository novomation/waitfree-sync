use waitfree_sync::{spsc::SendError, *};

pub trait ReadPrimitive<T> {
    fn read(&mut self) -> Option<T>
    where
        T: Clone;
}
pub trait WritePrimitive<T, E> {
    fn write(&mut self, data: T) -> Result<(), E>;
}

impl<T> ReadPrimitive<T> for triple_buffer::Reader<T> {
    #[inline]
    fn read(&mut self) -> Option<T>
    where
        T: Clone,
    {
        self.try_read()
    }
}

impl<T> WritePrimitive<T, ()> for triple_buffer::Writer<T> {
    fn write(&mut self, data: T) -> Result<(), ()> {
        self.write(data);
        Ok(())
    }
}

impl<T> ReadPrimitive<T> for spsc::Receiver<T> {
    #[inline]
    fn read(&mut self) -> Option<T>
    where
        T: Clone,
    {
        self.try_recv().ok()
    }
}

impl<T> WritePrimitive<T, SendError<T>> for spsc::Sender<T> {
    fn write(&mut self, data: T) -> Result<(), SendError<T>> {
        self.try_send(data)
    }
}
