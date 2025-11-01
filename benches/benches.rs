use criterion::{black_box, criterion_group, criterion_main, BatchSize, Bencher, BenchmarkId, Criterion};
use rand::seq::SliceRandom;
use simple_tree::SimpleTree;
use std::{collections::BTreeSet, time::Duration};

const INSERT_SIZE: u32 = 100_000;

fn benchmark_insert_st(b: &mut Bencher, size: usize) {
    b.iter(|| {
        let mut st = SimpleTree::<u32>::default();
        for _ in 0..size {
            st.insert(black_box(rand::random()), ());
        }
    });
}

fn benchmark_insert_bt(b: &mut Bencher, size: usize) {
    b.iter(|| {
        let mut bt = BTreeSet::<u32>::new();
        for _ in 0..size {
            bt.insert(black_box(rand::random()));
        }
    });
}

fn benchmark_range_st(b: &mut Bencher, test_size: usize, st: &SimpleTree<u32>) {
    b.iter(|| {
        let a: Vec<_> = (0..test_size as u32)
            .map(|n| n * 10)
            .map(|from| {
                let to = from + 1000;
                st.range(from..to).map(|&(k,_)| k).sum::<u32>()
            })
            .collect();
        black_box(a);
    });
}

fn benchmark_range_bt(b: &mut Bencher, test_size: usize, bt: &BTreeSet<u32>) {
    b.iter(|| {
        let b: Vec<_> = (0..test_size as u32)
            .map(|n| n * 10)
            .map(|from| {
                let to = from + 1000;
                bt.range(from..to).cloned().sum::<u32>()
            })
            .collect();
        black_box(b);
    });
}

fn benchmark_remove_st(b: &mut Bencher, st: &SimpleTree<u32>) {
    let mut keys: Vec<u32> = (0..1000).collect();
    keys.shuffle(&mut rand::thread_rng());

    b.iter_batched(|| {
        st.clone()
    }, |mut tree| {
        for key in &keys {
            tree.remove(black_box(key));
        }
    }, BatchSize::SmallInput);
}
fn benchmark_remove_bt(b: &mut Bencher, bt: &BTreeSet<u32>) {
    let mut keys: Vec<u32> = (0..1000).collect();
    keys.shuffle(&mut rand::thread_rng());

    b.iter_batched(|| {
        bt.clone()
    }, |mut btree_set| {
        for key in &keys {
            btree_set.remove(black_box(key));
        }
    }, BatchSize::SmallInput);
}

fn criterion_benchmark(c: &mut Criterion) {
    {
        let mut c = c.benchmark_group("insert");
        c.warm_up_time(Duration::from_millis(100));
        c.measurement_time(Duration::from_secs(1));
        for size in [1_000, 2_000, 3_000, 5_000,  10_000] {
            c.bench_with_input(BenchmarkId::new("btree", size), &size, |c, s| benchmark_insert_bt(c, *s));
            c.bench_with_input(BenchmarkId::new("simple_tree", size), &size, |c, s| benchmark_insert_st(c, *s));
        }
    }
    {
        let mut c = c.benchmark_group("range");
        c.warm_up_time(Duration::from_millis(100));
        c.measurement_time(Duration::from_secs(1));
        for size in [1_000, 2_000, 3_000, 5_000,  10_000] {
            let mut bt = BTreeSet::<u32>::new();
            for _ in 0..INSERT_SIZE {
                bt.insert(rand::random());
            };
            let mut st: SimpleTree<u32> = SimpleTree::default();
            for _ in 0..INSERT_SIZE {
                st.insert(rand::random(), ());
            }

            c.bench_with_input(BenchmarkId::new("btree", size), &size, |c, s| benchmark_range_bt(c, *s, &bt));
            c.bench_with_input(BenchmarkId::new("simple_tree", size), &size, |c, s| benchmark_range_st(c, *s, &st));
        }
    }
    {
        let mut c = c.benchmark_group("remove");
        c.warm_up_time(Duration::from_millis(100));
        c.measurement_time(Duration::from_secs(1));
        for size in [10_000, 20_000, 30_000, 50_000] {
            let mut bt = BTreeSet::<u32>::new();
            for _ in 0..INSERT_SIZE {
                bt.insert(rand::random());
            };
            let mut st: SimpleTree<u32> = SimpleTree::default();
            for _ in 0..INSERT_SIZE {
                st.insert(rand::random(), ());
            }

            c.bench_with_input(BenchmarkId::new("btree", size), &size, |c, _| benchmark_remove_bt(c, &bt));
            c.bench_with_input(BenchmarkId::new("simple_tree", size), &size, |c, _| benchmark_remove_st(c, &st));
        }
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
