# A Simple BTree Implementation
This is a simple BTree implementation in 100 lines of Rust code.

Its inserts and removals are on par with the standard library, it's a bit slower and that might be because this is a top down BTree.

The range queries, though, having been implemented with coroutines, and is 2 to 3 times slower than the standard library BTree.