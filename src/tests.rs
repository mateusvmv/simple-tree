use rand::seq::SliceRandom;

use super::*;

fn in_order_traversal(tree: &SimpleTree<u32>) -> Vec<u32> {
    tree.range(..).map(|(k,_)| *k).collect()
}

#[test]
fn test_remove() {
    let mut tree = SimpleTree::default();

    let keys = vec![10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120, 130, 140, 150];
    for &key in &keys {
        tree.insert(key, ());
    }

    tree.remove(&150);
    assert_eq!(in_order_traversal(&tree), vec![10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120, 130, 140]);

    tree.remove(&70);
    assert_eq!(in_order_traversal(&tree), vec![10, 20, 30, 40, 50, 60, 80, 90, 100, 110, 120, 130, 140]);

    tree.remove(&999);
    assert_eq!(in_order_traversal(&tree), vec![10, 20, 30, 40, 50, 60, 80, 90, 100, 110, 120, 130, 140]);

    tree.remove(&40);
    assert_eq!(in_order_traversal(&tree), vec![10, 20, 30, 50, 60, 80, 90, 100, 110, 120, 130, 140]);

    for &key in &[10, 20, 30, 50, 60, 80, 90, 100, 110, 120, 130, 140] {
        tree.remove(&key);
    }
}

use rand::Rng;

#[test]
fn test_range() {
    let mut tree = SimpleTree::default();
    let mut keys: Vec<i32> = (0..20).map(|_| rand::random::<i32>()).collect();
    keys.sort();
    keys.dedup();
    for &k in &keys {
        tree.insert(k, ());
    }
    let collect_tree_keys = |start: Bound<i32>, end: Bound<i32>| -> Vec<i32> {
        tree.range((start, end))
            .map(|(k, _)| *k)
            .collect::<Vec<_>>()
    };

    let mut rng = rand::thread_rng();
    for _ in 0..100 {
        let a = keys[rng.gen_range(0..keys.len())];
        let b = keys[rng.gen_range(0..keys.len())];
        let (start, end) = if a <= b { (a, b) } else { (b, a) };

        let expected: Vec<i32> = keys
            .iter()
            .cloned()
            .filter(|k| *k >= start && *k < end)
            .collect();

        let result = collect_tree_keys(Bound::Included(start), Bound::Excluded(end));

        assert_eq!(
            result, expected,
            "range({start:?}..{end:?}) mismatch",
        );
    }
    let full: Vec<i32> = tree.range(..).map(|(k, _)| *k).collect();
    assert_eq!(full, keys);
}

#[test]
fn test_random() {
    let mut tree = SimpleTree::default();
    let mut keys: Vec<i32> = vec![];
    for _ in 0..10_000 {
        keys.push(rand::random());
    }
    keys.sort();
    keys.dedup();
    let mut rng = rand::thread_rng();
    keys.shuffle(&mut rng);

    for &k in &keys {
        tree.insert(k, ());
        assert!(tree.remove(&k).is_some());
        tree.insert(k, ());
    }
    eprintln!("{:?}", tree.range(..).collect::<Vec<_>>());
    for k in &keys {
        assert!(tree.get(k).is_some());
        assert!(tree.remove(k).is_some());
    }
    assert!(tree.range(..).next().is_none());
}