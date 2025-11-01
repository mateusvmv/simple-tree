# A Simple B-Tree Implementation

This document describes a simple B-Tree implementation in approximately 130 lines of Rust.  
It is constructed from functions that mimic movements of keys in a 2D plane (see the [Building Blocks](#building-blocks) section).  

The tree performs **preemptive inserts** (splitting before descending), while removals involve an **ascent phase**.  
It maintains the following invariants:

- Each node has between `A` and `B = 2A + 1` keys, and may temporarily hold `B + 1` during merge operations.  
- A node is either a **leaf** or has exactly `|keys| + 1` children.  
- The key at index `i` is strictly greater than all keys in child `i`.  
- The final child (index `|keys|`) contains all elements greater than the last key.  

These size constraints ensure that the tree remains **perfectly balanced**.

Performance characteristics:
- Insertions are on par with the Rust standard library, slightly (~1/6th) faster.
- Removals are around **3× faster** than STL.
- Range queries are **2× faster** than STL.

---

## Implementation

A tree node consists of a list of `(key, value)` pairs and an optional list of children:
Each node stores its entries (`keys` and associated `values`) and, if it’s not a leaf, its list of child nodes.
```rs
Node {
    entries: Vec<(Key, Value)>,
    children: Option<Vec<Node>,
}
```

Or equivalently, the entire tree can be represented as a tuple:
```rs
Tree(Vec<(Key, Value)>, Option<Vec<Node>>)
```

---

## Building Blocks

The two main operations that mutate the tree are **upgrade** and **downgrade**.  
They conceptually move keys *up* or *down* between tree levels, like movements in a 2D plane:

A key moved *up* goes towards the root, and a key moved *down* goes towards the leaves.
In this analogy, we are using the upper layer keys as separators, so upgrading a key makes it a separator and moves other entries on its node.
Downgrading a key, on the other hand, merges two nodes in the layer below.

---

Consider a B-Tree storing integers from 0–9, with `A = 2` and `B = 5`:
```rs

// First 5 inserts fit in one node

0 1 2 3 4

// Next insert triggers a preemptive split (upgrading 2)

    2
0 1   3 4

// Further inserts fill until another split (upgrading 5)

    2
0 1   3 4 5 6 8

// Final structure:

    2     5
0 1   3 4   6 7 8 9

```

---

### Upgrade Function

`upgrade` promotes one of its keys to the parent as a separator.
The child is split into two halves: left and right. The left child willl end up
with all entries before the selected grandchild, and the right half will end up with
all entries past the selected grandchild. That means `grandchild_index` becomes
the length of the new left child. The separator and the right child are inserted at index
`child_index`, so all indexes past them are moved to the right.

```rs

fn upgrade(entries, children, child_index, grandchild_index) {
    let left_child = entry in children at child_index
    let promoted_key = extract entry in left_child at grandchild_index
    let right_half = extract entries and children from left_child starting at grandchild_index + 1

    insert in children, at child_index + 1, the right_half
    insert in entries, at child_index, the promoted_key

    let ci = grandchildren_index
    at the end, left_child must have ci entries and ci + 1 children.
}

```

---

### Downgrade Function

`downgrade` merges a child with its right sibling.
It pulls down the separating key from the parent and concatenates both children’s keys and subtrees.
It takes the separator from index `child_index`, and the right child from index `child_index + 1`, so
entries past those indices are moved to the left. The left child, at index `child_index`, receives the
separator and the removed entries from the right child.

```rs
fn downgrade(keys, children, child_index) {
    let right_child = remove from children at child_index + 1
    let separator = remove from keys at child_index

    let left_child = children at child_index
    append separator to left_child entries
    extend left_child entries with righ_child entries
    if left_child has children:
        extend left_child children with righ_child children
}
```

---

## Borrowing

During **removal**, before descending into a child node, we must ensure it has more than `A` keys.  
If not, we use `borrow` to rebalance — either by:
1. Borrowing a key from a sibling that has extra keys, or
2. Merging with a sibling if both are at minimum capacity.

`borrow` prefers merging or borrowing from the **right sibling**, although it makes little difference whether to prefer the
left or right siblings. When borrowing, it may need to call `upgrade` to split a sibling node so that each child has a valid number of elements,
and that means the parent node could temporarily have `B+1` elements, if it starts the function with `B` elements.
Finally, it merges nodes using `downgrade`, which reduces the size of the parent node once again.

```rs

fn borrow(entries, children, child_index) {
    if children[child_index].entries.size > A:
        return

    // Try right sibling
    if has right sibling:
        if right_sibling has |entries| > A:
            let grandchild_index = right_sibling |entries| - A - 1
            upgrade(entries, children, child_index + 1, grandchild_index)
            // right sibling will end up with size |entries| - A - 1
            // we downgrade and get size A + |right_sibling| + 1 = |entries| > A
            downgrade(entries, children, child_index)
        else:
            // right_sibling has A entries, so we downgrade and get size 2 * A + 1 = B > A
            downgrade(entries, children, child_index)

    // Else, try left sibling
    else if left_sibling has |entries| <= A:
        // left_sibling has size A, so we downgrade its key to merge with the next sibling
        // it can't have |entries| < A due to our invariants
        decrement child_index
        // we get size 2 * A + 1 = B
        downgrade(entries, children, child_index)
    else:
        // left sibling has more than A entries, we split the extra
        upgrade(entries, children, child_index - 1, A)
        // entry at child_index will have |entries| - A elements
        // we know that entry at child_index + 1 has A elements, it's our previous child
        // we downgrande and get size A + |entries| - A + 1 = |entries| + 1 > A
        downgrade(entries, children, child_index)

    return updated child_index
}
```

---

## Insertion and Deletion

### Insertion

Our implementation of `insert` is **preemptive** — it will attempt to split full nodes *before* descending.
Whenever we descend, we'll find the correct child with a lower bound on the keys. That is, we find the first key greater than or equal to the desired key, and its index will also give us the index of the child node to descend into. If the key is ever equal, we stop descent on inserts and update the value.

For the inserts, on a node with `B` elements, it will call upgrade with `gc = A`, and that makes two nodes of A elements and one separator, totaling `2 * A + 1 = B`. Since upgrade will create a new separator at `ci`, we might need to move `ci` up by `1`.

```rs

fn insert(node, key, value) {
    index = first index where key of node.entries at index >= key
    
    if equal key exists:
        update value
        return
    
    if node is leaf:
        insert (key, value) at index
        return
    
    // Preemptive split
    if |entries| of node.children at index > B:
        upgrade(node.entries, node.children, index, A)
        if key > node.entries[index].key:
            index += 1

    insert(node.children[index], key, value)
}

```

---

### Deletion

For the removals, it'll descend to the correct node, calling the `borrow` function before descent to ensure that the target child has more than `A` entries.
If it finds itself in a leaf, it removes the key if it exists, or does nothing otherwise. On the other hand, if it isn't yet in a leaf node, it'll descend until it finds
the query key or a leaf node. If it encounters the desired key it descends into the largest child repeatedly, while calling `borrow` to maintain the invariants.
At the end of this descent, it removes the last entry, places it where our found entry was located, and returns the found entry.

```rs
fn remove(node, key) {
    index = first index where key of node.entries at index >= key

    if node is leaf:
        if node.entries[index].key == key:
            node.entries.remove(index)
        return

    borrow(node.entries, node.children, index)

    let found_entry
    if key of entry in node at index == key:
        // Replace with predecessor
        pred_node = node.children at index
        while pred_node is not leaf:
            borrow(pred_node.entries, pred_node.children, pred_node.children.size - 1)
            pred_node = last of pred_node.children
        found_entry = node.entries at index
        node.entries at index = remove last of pred_node.entries
        return found_entry

    if node has zero entries and one child:
        node = child at index 0
        return remove(node, key)

    return remove(node.children[index], key)
}

```

---

## Range Query

The range query iteratively traverses the tree in **in-order**, returning all elements within a specified `[min, max]` range.  
It maintains a **stack** of traversal states `(node, index, optional after_entry)`.

We'll always look at the last tuple in the stack and increase `i` while the entry at index `i` is to the left of our desired range, then look at the node.

If the node at the top of the stack is a branch node, we must push a new tuple into the stack containing the child at index `i`, the index `0` and, if it falls within the desired range, the entry at index `i` as the after entry. After pushing the child, we increment the index `i` on the parent. That means we'll always traverse the children before the parents.

If, however, we're on a leaf, and the entry at index `i` is within the desired range, then we return it and increment `i`. If we reach the end of the entries, be it by reaching the last entry or falling outside of the desired range, we pop the top of the stack and return `after_entry` instead.

One can see how this pattern will ensure that our stack is bounded by `O(log_B(n))` on its size, how it traverses the BTree in-order, and prunes children that aren't meant to fit in the query range.

```rs
fn range_query(tree, min_key, max_key) {
    let stack = empty stack
    push tuple (tree, 0, None) to the stack
    
    while stack is not empty:
        let (node, i, aft) = top of stack

        // Skip keys < min_key
        while i < node.entries.size and node.entries[i].key < min_key:
            i += 1
        
        if node has children:
            let next_child = node.children at i
            let next_aft = node.entries at i if key of node.entries at i <= max_key else None
            increment second value (i) of the top of the stack
            push tuple (next_child, 0, next_aft) to stack
            continue

        entry = node.entries at i
        if entry exists and key of entry <= max_key:
            increment second value (i) of the top of the stack
            yield entry
            continue

        pop last value of stack
        if aft != None and key of aft <= max_key:
            yield aft
            continue
}

```
