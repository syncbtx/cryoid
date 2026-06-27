use cryoid::{decode, layouts::Snowflake63, Gen, Layout};
use divan::{black_box, Bencher};

fn main() {
    divan::main();
}


#[divan::bench]
fn next_id(bencher: Bencher) {
    bencher
        .counter(1usize) 
        .with_inputs(|| {
            let mut engine = Gen::<Snowflake63>::builder().machine(1).build().unwrap();
            let ts = Snowflake63::EPOCH + 1;
            engine.next_id(ts).unwrap();
            (engine, ts)
        })
        .bench_values(|(mut engine, ts)| {
            engine.next_id(ts).unwrap()
        });
}


#[divan::bench(args = [10, 100, 1000, 4000])]
fn batch_reservation(bencher: Bencher, size: u64) {
    bencher
        .counter(1usize)
        .with_inputs(|| {
            let engine = Gen::<Snowflake63>::builder().machine(1).build().unwrap();
            (engine, Snowflake63::EPOCH + 1)
        })
        .bench_values(|(mut engine, ts)| {
            engine.batch(ts, size).unwrap()
        });
}


#[divan::bench(args = [10, 100, 1000])]
fn batch_consume_forward(bencher: Bencher, size: u64) {
    bencher
        // Tells Divan we are yielding `size` items, unlocking the Gitem/s metric
        .counter(size as usize)
        .with_inputs(|| {
            let mut engine = Gen::<Snowflake63>::builder().machine(1).build().unwrap();
            engine.batch(Snowflake63::EPOCH + 1, size).unwrap()
        })
        .bench_values(|batch| {
            for id in batch {
                black_box(id);
            }
        });
}


#[divan::bench(args = [10, 100, 1000])]
fn batch_consume_reverse(bencher: Bencher, size: u64) {
    bencher
        .counter(size as usize)
        .with_inputs(|| {
            let mut engine = Gen::<Snowflake63>::builder().machine(1).build().unwrap();
            engine.batch(Snowflake63::EPOCH + 1, size).unwrap()
        })
        .bench_values(|batch| {
            for id in batch.rev() {
                black_box(id);
            }
        });
}


#[divan::bench]
fn decode_snowflake(bencher: Bencher) {
    let raw_id = 326101123750432768;

    bencher
        .counter(1usize)
        .bench(|| {
            decode::<Snowflake63>(black_box(raw_id))
        });
}