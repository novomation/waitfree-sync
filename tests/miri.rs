use common::{ReadPrimitive, WritePrimitive};
#[cfg(loom)]
use loom::thread;
use std::fmt::Debug;
#[cfg(not(loom))]
use std::thread;
use waitfree_sync::spsc;
use waitfree_sync::triple_buffer;

mod common;

#[cfg(loom)]
const COUNT: usize = 4;
#[cfg(all(not(loom), not(miri)))]
const COUNT: usize = 16_384;
#[cfg(miri)]
const COUNT: usize = 1024;

type Payload = [i32; 50];
fn test_multithread<E: PartialEq + Debug>(
    reader_writer: (
        impl WritePrimitive<Payload, E> + Send + Sync + 'static,
        impl ReadPrimitive<Payload> + Send + Sync + 'static,
    ),
) {
    let (mut writer, mut reader) = reader_writer;
    assert_eq!(writer.write([1; 50]), Ok(()));
    assert_eq!(reader.read(), Some([1; 50]));

    let writer_thread = thread::spawn(move || {
        thread::park();
        for i in 0..COUNT {
            assert_eq!(writer.write([i as i32; 50]), Ok(()));
        }
    });
    let reader_thread = thread::spawn(move || {
        thread::park();
        for _ in 0..COUNT {
            if let Some(val) = reader.read() {
                let first_entry = val[0];
                for entry in val {
                    assert_eq!(entry, first_entry);
                }
            }
        }
    });
    writer_thread.thread().unpark();
    reader_thread.thread().unpark();
    assert!(writer_thread.join().is_ok());
    assert!(reader_thread.join().is_ok());
}

#[derive(Debug, PartialEq, Clone)]
pub struct SomeStruct {
    pub counter: i32,
    pub inner_field: Vec<Option<SomeEnum>>,
}
impl Default for SomeStruct {
    fn default() -> Self {
        Self {
            counter: 0,
            inner_field: vec![Some(SomeEnum::State1)],
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum SomeEnum {
    State1,
    State2,
    State3,
    State4,
    State5,
}

fn test_heapdata<E: PartialEq + Debug>(
    reader_writer: (
        impl WritePrimitive<SomeStruct, E> + Send + Sync + 'static,
        impl ReadPrimitive<SomeStruct> + Send + Sync + 'static,
    ),
) {
    let (mut writer, mut reader) = reader_writer;
    assert_eq!(writer.write(SomeStruct::default()), Ok(()));
    assert_eq!(reader.read(), Some(SomeStruct::default()));
}

fn test_heapdata_multithread<E: PartialEq + Debug>(
    reader_writer: (
        impl WritePrimitive<SomeStruct, E> + Send + Sync + 'static,
        impl ReadPrimitive<SomeStruct> + Send + Sync + 'static,
    ),
) {
    let (mut writer, mut reader) = reader_writer;
    assert_eq!(writer.write(SomeStruct::default()), Ok(()));
    assert_eq!(reader.read(), Some(SomeStruct::default()));
    let writer_thread = thread::spawn(move || {
        thread::park();
        for i in 0..COUNT {
            assert_eq!(
                writer.write(SomeStruct {
                    counter: i as i32,
                    inner_field: vec![Some(SomeEnum::State1)]
                }),
                Ok(())
            );
        }
    });
    let reader_thread = thread::spawn(move || {
        thread::park();
        for _ in 0..COUNT {
            if let Some(val) = reader.read() {
                assert_eq!(val.inner_field, vec![Some(SomeEnum::State1)]);
            }
        }
    });
    writer_thread.thread().unpark();
    reader_thread.thread().unpark();
    assert!(writer_thread.join().is_ok());
    assert!(reader_thread.join().is_ok());
}

#[cfg(not(loom))]
#[test]
fn test_tripple_buffer() {
    test_multithread(triple_buffer::triple_buffer());
    test_heapdata(triple_buffer::triple_buffer());
    test_heapdata_multithread(triple_buffer::triple_buffer());
}

#[cfg(not(loom))]
#[test]
fn test_spsc() {
    test_multithread(spsc::spsc(COUNT));
    test_heapdata(spsc::spsc(COUNT));
    test_heapdata_multithread(spsc::spsc(COUNT));
}

#[test]
#[cfg(loom)]
fn loom_tripple_buffer() {
    loom::model(|| {
        test_multithread(triple_buffer::triple_buffer());
    });

    loom::model(|| {
        test_heapdata(triple_buffer::triple_buffer());
    });

    loom::model(|| {
        test_heapdata_multithread(triple_buffer::triple_buffer());
    });
}

#[test]
#[cfg(loom)]
fn loom_spsc() {
    loom::model(|| {
        test_multithread(spsc::spsc::<_, COUNT>());
    });
    loom::model(|| {
        test_heapdata(spsc::spsc::<_, COUNT>());
    });
    loom::model(|| {
        test_heapdata_multithread(spsc::spsc::<_, COUNT>());
    });
}
