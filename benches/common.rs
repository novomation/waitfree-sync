use std::sync::mpsc;
use waitfree_sync::{spsc, triple_buffer};

pub trait ReadPrimitive<T: Send>: Send + 'static {
    fn read(&mut self) -> Option<T>
    where
        T: Clone;
}
pub trait WritePrimitive<T: Send>: Send + 'static {
    #[allow(clippy::result_unit_err)]
    fn write(&mut self, data: T) -> Result<(), ()>;
}

pub trait New: Send + 'static {
    fn new_channel<T: Send + 'static>(
        &self,
        size: usize,
    ) -> (impl WritePrimitive<T>, impl ReadPrimitive<T>);
    fn name(&self) -> &str;
}

pub struct ChSpsc;
impl New for ChSpsc {
    fn new_channel<T: Send + 'static>(
        &self,
        size: usize,
    ) -> (impl WritePrimitive<T>, impl ReadPrimitive<T>) {
        spsc::spsc(size)
    }
    fn name(&self) -> &str {
        "SPSC"
    }
}
pub struct ChMpsc;
impl New for ChMpsc {
    fn new_channel<T: Send + 'static>(
        &self,
        size: usize,
    ) -> (impl WritePrimitive<T>, impl ReadPrimitive<T>) {
        mpsc::sync_channel(size)
    }
    fn name(&self) -> &str {
        "StdMPSC"
    }
}
pub struct ChFlume;
impl New for ChFlume {
    fn new_channel<T: Send + 'static>(
        &self,
        size: usize,
    ) -> (impl WritePrimitive<T>, impl ReadPrimitive<T>) {
        flume::bounded(size)
    }
    fn name(&self) -> &str {
        "Flume"
    }
}
pub struct ChCrossbeam;
impl New for ChCrossbeam {
    fn new_channel<T: Send + 'static>(
        &self,
        size: usize,
    ) -> (impl WritePrimitive<T>, impl ReadPrimitive<T>) {
        crossbeam_channel::bounded(size)
    }
    fn name(&self) -> &str {
        "Crossbeam"
    }
}

// --- mpsc

impl<T: Send + 'static> ReadPrimitive<T> for mpsc::Receiver<T> {
    #[inline]
    fn read(&mut self) -> Option<T>
    where
        T: Clone,
    {
        self.try_recv().ok()
    }
}

impl<T: Send + 'static> WritePrimitive<T> for mpsc::SyncSender<T> {
    fn write(&mut self, data: T) -> Result<(), ()> {
        self.try_send(data).or(Err(()))
    }
}

// --- crossbeam
impl<T: Send + 'static> ReadPrimitive<T> for crossbeam_channel::Receiver<T> {
    #[inline]
    fn read(&mut self) -> Option<T>
    where
        T: Clone,
    {
        self.try_recv().ok()
    }
}

impl<T: Send + 'static> WritePrimitive<T> for crossbeam_channel::Sender<T> {
    fn write(&mut self, data: T) -> Result<(), ()> {
        self.try_send(data).or(Err(()))
    }
}

// --- flume
impl<T: Send + 'static> ReadPrimitive<T> for flume::Receiver<T> {
    #[inline]
    fn read(&mut self) -> Option<T>
    where
        T: Clone,
    {
        self.try_recv().ok()
    }
}

impl<T: Send + 'static> WritePrimitive<T> for flume::Sender<T> {
    fn write(&mut self, data: T) -> Result<(), ()> {
        self.try_send(data).or(Err(()))
    }
}

// --- triple_buffer
impl<T: Send + 'static> ReadPrimitive<T> for triple_buffer::Reader<T> {
    #[inline]
    fn read(&mut self) -> Option<T>
    where
        T: Clone,
    {
        self.try_read()
    }
}

impl<T: Send + 'static> WritePrimitive<T> for triple_buffer::Writer<T> {
    fn write(&mut self, data: T) -> Result<(), ()> {
        self.write(data);
        Ok(())
    }
}

// --- SPSC
impl<T: Send + 'static> ReadPrimitive<T> for spsc::Receiver<T> {
    #[inline]
    fn read(&mut self) -> Option<T>
    where
        T: Clone,
    {
        self.try_recv()
    }
}

impl<T: Send + 'static> WritePrimitive<T> for spsc::Sender<T> {
    fn write(&mut self, data: T) -> Result<(), ()> {
        self.try_send(data).or(Err(()))
    }
}
