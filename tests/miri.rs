mod common;
use common::{ReadPrimitive, WritePrimitive};
use waitfree_sync::triple_buffer;

#[cfg(loom)]
use loom::thread;
use std::fmt::Debug;
#[cfg(not(loom))]
use std::thread;

type Payload = [i32; 50];
fn generic_test<E: PartialEq + Debug>(
    reader_writer: (
        impl ReadPrimitive<Payload> + Send + Sync + 'static,
        impl WritePrimitive<Payload, E> + Send + Sync + 'static,
    ),
) {
    let (mut reader, mut writer) = reader_writer;
    assert_eq!(writer.write([1; 50]), Ok(()));
    assert_eq!(reader.read(), Some([1; 50]));
    #[cfg(loom)]
    const COUNT: i32 = 4;
    #[cfg(all(not(loom), not(miri)))]
    const COUNT: i32 = 10_000;
    #[cfg(miri)]
    const COUNT: i32 = 300;
    let writer_thread = thread::spawn(move || {
        thread::park();
        for i in 0..COUNT {
            assert_eq!(writer.write([i; 50]), Ok(()));
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

#[cfg(not(loom))]
#[test]
fn test_tripple_buffer() {
    generic_test(triple_buffer::triple_buffer());
}

#[test]
#[cfg(loom)]
fn test_tripple_buffer() {
    let mut loom_rt = loom::model::Builder::new();
    // loom_rt.max_threads = 2;
    loom_rt.max_branches = 100_000;
    loom_rt.check(|| {
        generic_test(triple_buffer::triple_buffer());
    });
}
