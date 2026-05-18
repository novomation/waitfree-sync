use core_affinity::CoreId;
use criterion::{criterion_group, criterion_main, Criterion};
use std::{
    hint::black_box,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Barrier,
    },
    time::Instant,
};
use waitfree_sync::spsc;

fn test_threaded_single_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("threaded_single_write");
    group.bench_function("spsc", move |b| {
        let (mut tx, mut rx) = spsc::spsc(32_768);
        let barrier = Arc::new(Barrier::new(2));
        let stop = Arc::new(AtomicBool::new(false));

        let rx_thread = std::thread::spawn({
            let barrier = barrier.clone();
            let stop = stop.clone();
            move || {
                core_affinity::set_for_current(CoreId { id: 1 });
                barrier.wait(); // wait for benchmark to start
                while !stop.load(Ordering::Relaxed) {
                    let _ = rx.try_recv();
                }
            }
        });
        core_affinity::set_for_current(CoreId { id: 2 });
        // synchronize with consumer
        barrier.wait();
        b.iter(move || {
            let start = Instant::now();
            for _ in 0..1024 {
                let _ = black_box(tx.try_send(5614));
            }
            start.elapsed()
        });
        stop.store(true, Ordering::Relaxed);
        rx_thread.join().unwrap();
    });
}

fn test_threaded_single_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("threaded_single_read");
    group.bench_function("spsc", move |b| {
        let (mut tx, mut rx) = spsc::spsc(32_768);
        let barrier = Arc::new(Barrier::new(2));
        let stop = Arc::new(AtomicBool::new(false));

        let rx_thread = std::thread::spawn({
            let barrier = barrier.clone();
            let stop = stop.clone();
            move || {
                core_affinity::set_for_current(CoreId { id: 1 });
                barrier.wait(); // wait for benchmark to start
                while !stop.load(Ordering::Relaxed) {
                    let _ = tx.try_send(5614);
                }
            }
        });
        core_affinity::set_for_current(CoreId { id: 2 });
        // synchronize with consumer
        barrier.wait();
        b.iter(move || {
            let start = Instant::now();
            for _ in 0..1024 {
                black_box(rx.try_recv());
            }
            start.elapsed()
        });
        stop.store(true, Ordering::Relaxed);
        rx_thread.join().unwrap();
    });
}

fn threaded_single_write(c: &mut Criterion) {
    test_threaded_single_write(c);
}

fn threaded_single_read(c: &mut Criterion) {
    test_threaded_single_read(c);
}

criterion_group!(benches, threaded_single_write, threaded_single_read,);
criterion_main!(benches);
