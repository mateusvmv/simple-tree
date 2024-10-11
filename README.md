# A Simple BTree Implementation
This is a simple BTree implementation in 100 lines of Rust code.

Its inserts and removals are on par with the standard library, it's a bit slower and that might be because this is a top down BTree.

The range queries, though, having been implemented with coroutines and using boxing at each step, due to recursion, are 5 to 6 tiems slower than the standard library BTree. It could have been implemented by actually managing the stack, but then the code size would increase, and the goal of this little project was to implemnent a BTree with as few LOC as possible.