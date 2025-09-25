use crate::common::{ChCrossbeam, ChFlume, ChMpsc, ChSpsc, New, ReadPrimitive, WritePrimitive};
use criterion::{criterion_group, criterion_main, Criterion};
use std::{
    hint::black_box,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Barrier,
    },
    time::Instant,
};

mod common;

fn test_threaded_single_write(c: &mut Criterion, channel: impl New) {
    let mut group = c.benchmark_group("threaded_single_write");
    group.bench_function(channel.name().to_string().as_str(), move |b| {
        let (mut tx, mut rx) = channel.new_channel(32_768);
        let barrier = Arc::new(Barrier::new(2));
        let stop = Arc::new(AtomicBool::new(false));

        let rx_thread = std::thread::spawn({
            let barrier = barrier.clone();
            let stop = stop.clone();
            move || {
                affinity::set_thread_affinity([1]).unwrap();
                barrier.wait(); // wait for benchmark to start
                while !stop.load(Ordering::Relaxed) {
                    let _ = rx.read();
                }
            }
        });
        affinity::set_thread_affinity([2]).unwrap();
        // synchronize with consumer
        barrier.wait();
        b.iter(move || {
            let start = Instant::now();
            for _ in 0..1024 {
                let _ = black_box(tx.write(5614));
            }
            start.elapsed()
        });
        stop.store(true, Ordering::Relaxed);
        rx_thread.join().unwrap();
    });
}

fn test_threaded_single_read(c: &mut Criterion, channel: impl New) {
    let mut group = c.benchmark_group("threaded_single_read");
    group.bench_function(channel.name().to_string().as_str(), move |b| {
        let (mut tx, mut rx) = channel.new_channel(32_768);
        let barrier = Arc::new(Barrier::new(2));
        let stop = Arc::new(AtomicBool::new(false));

        let rx_thread = std::thread::spawn({
            let barrier = barrier.clone();
            let stop = stop.clone();
            move || {
                affinity::set_thread_affinity([1]).unwrap();
                barrier.wait(); // wait for benchmark to start
                while !stop.load(Ordering::Relaxed) {
                    let _ = tx.write(5614);
                }
            }
        });
        affinity::set_thread_affinity([2]).unwrap();
        // synchronize with consumer
        barrier.wait();
        b.iter(move || {
            let start = Instant::now();
            for _ in 0..1024 {
                black_box(rx.read());
            }
            start.elapsed()
        });
        stop.store(true, Ordering::Relaxed);
        rx_thread.join().unwrap();
    });
}

fn threaded_single_write(c: &mut Criterion) {
    test_threaded_single_write(c, ChSpsc);
    test_threaded_single_write(c, ChMpsc);
    test_threaded_single_write(c, ChFlume);
    test_threaded_single_write(c, ChCrossbeam);
}

fn threaded_single_read(c: &mut Criterion) {
    test_threaded_single_read(c, ChSpsc);
    test_threaded_single_read(c, ChMpsc);
    test_threaded_single_read(c, ChFlume);
    test_threaded_single_read(c, ChCrossbeam);
}

criterion_group!(benches, threaded_single_write, threaded_single_read,);
criterion_main!(benches);
