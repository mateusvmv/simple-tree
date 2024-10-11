use criterion::{black_box, criterion_group, criterion_main, Criterion};
use simple_tree::SimpleTree;
use std::collections::BTreeSet;

const INSERT_SIZE: u32 = 100_000;

fn benchmark_insert_st(c: &mut Criterion) {
    c.bench_function("SimpleTree Insert", |b| {
        b.iter(|| {
            let mut st = SimpleTree::<u32>::default();
            for _ in 0..INSERT_SIZE {
                st.insert(black_box(rand::random()), ());
            }
        });
    });
}

fn benchmark_insert_bt(c: &mut Criterion) {
    c.bench_function("BTreeSet Insert", |b| {
        b.iter(|| {
            let mut bt = BTreeSet::<u32>::new();
            for _ in 0..INSERT_SIZE {
                bt.insert(black_box(rand::random()));
            }
        });
    });
}

fn benchmark_range_st(c: &mut Criterion) {
    const TEST_SIZE: u32 = INSERT_SIZE / 10;

    let mut st = SimpleTree::default();
    for _ in 0..INSERT_SIZE {
        st.insert(rand::random(), ());
    }

    c.bench_function("SimpleTree Range Sum", |b| {
        b.iter(|| {
            let a: Vec<u32> = (0..TEST_SIZE)
                .map(|n| n * 10)
                .map(|from| {
                    let to = from + 1000;
                    st.range(from..to).map(|&(k,_)| k).sum::<u32>()
                })
                .collect();
            black_box(a);
        });
    });
}

fn benchmark_range_bt(c: &mut Criterion) {
    const TEST_SIZE: u32 = INSERT_SIZE / 10;

    let mut bt = BTreeSet::<u32>::new();
    for _ in 0..INSERT_SIZE {
        bt.insert(rand::random());
    }

    c.bench_function("BTreeSet Range Sum", |b| {
        b.iter(|| {
            let b: Vec<u32> = (0..TEST_SIZE)
                .map(|n| n * 10)
                .map(|from| {
                    let to = from + 1000;
                    bt.range(from..to).cloned().sum::<u32>()
                })
                .collect();
            black_box(b);
        });
    });
}

fn benchmark_remove_st(c: &mut Criterion) {
    let keys: Vec<u32> = (0..1000).collect();
    let mut tree = SimpleTree::default();
    for &k in &keys { tree.insert(k, ()); }
    c.bench_function("SimpleTree remove", |b| {
        b.iter(|| {
            let mut tree = tree.clone();
            for key in &keys {
                tree.remove(black_box(key));
            }
        });
    });
}
fn benchmark_remove_bt(c: &mut Criterion) {
    let keys: Vec<u32> = (0..1000).collect();
    let btree_set: BTreeSet<u32> = keys.iter().copied().collect();
    c.bench_function("BTreeSet remove", |b| {
        b.iter(|| {
            let mut btree_set = btree_set.clone();
            for key in &keys {
                btree_set.remove(black_box(key));
            }
        });
    });
}

criterion_group!(
    benches,
    benchmark_insert_st,
    benchmark_insert_bt,
    benchmark_range_st,
    benchmark_range_bt,
    benchmark_remove_bt,
    benchmark_remove_st,
);
criterion_main!(benches);
