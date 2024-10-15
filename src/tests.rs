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

#[test]
fn test_random() {
    let mut tree = SimpleTree::default();
    let mut keys: Vec<i32> = vec![];
    for _ in 0..10_000 {
        keys.push(rand::random());
    }
    keys.sort();
    keys.dedup();
    for &k in &keys {
        tree.insert(k, ());
    }
    eprintln!("{:?}", tree.range(..).collect::<Vec<_>>());
    for k in &keys {
        eprintln!("remove {k} from {tree:?}");
        assert!(tree.remove(k).is_some());
    }
}