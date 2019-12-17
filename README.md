# Arena based tree structure

[![Build Status](https://travis-ci.com/macthecadillac/atree.svg?branch=master)](https://travis-ci.com/macthecadillac/atree)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

An arena based tree structure. Being an arena based tree, this structure is
implemented on top of `Vec` and as such eliminates the need for the countless
heap allocations or unsafe code that a pointer based tree structure would
require. This approach also makes parallel access feasible.  On top of the basic
node insertion and removal operations, care is taken to provide various
functions which enable splitting, merging, and numerous kinds of immutable and
mutable iterations over the nodes.

Most of the code in the crate is `unsafe` free, except for the mutable
iterators, where the `unsafe` code is lifted from the core Rust
implementation of `IterMut`.

## Quick Start

The crate consists of three main `struct`s: `Tree<T>`, `Token` and
`Node<T>`. `Tree<T>` provides the arena in which all data is stored.
The data can then be accessed by indexing `Tree<T>` with `Token`. `Node<T>`
is a container that encapsulates the data on the tree.

We can start by initializing an empty arena and add stuff to it at a later
time:
```rust
use atree::Tree;

let mut tree = Tree::default();
assert!(tree.is_empty());

// add stuff to the arena when we feel like it
let root_data = "Indo-European";
let root_node_token = tree.initialize(root_data);
assert_eq!(tree.node_count(), 1)
```

Another way is to directly initialize an arena with a node:
```rust
use atree::Tree;

let root_data = "Indo-European";
let (mut tree, root_token) = Tree::with_data(root_data);
assert_eq!(tree.node_count(), 1)
```

To add more data to the tree, call the `append` method on the tokens (we
can't do this directly to the nodes because of the limitations of borrow
checking).
```rust
use atree::Tree;

let root_data = "Indo-European";
let (mut tree, root_token) = Tree::with_data(root_data);
root_token.append(&mut tree, "Romance");
assert_eq!(tree.node_count(), 2);
```

To access/modify existing nodes in the tree, we can use indexing or
`get`/`get_mut`.
```rust
use atree::Tree;

let root_data = "Indo-European";
let (mut arena, root_token) = Tree::with_data(root_data);

// add some more stuff to the tree
let branch1 = root_token.append(&mut arena, "Romance");
let branch2 = root_token.append(&mut arena, "Germanic");
let branch3 = root_token.append(&mut arena, "Slavic");
let lang1 = branch2.append(&mut arena, "English");
let lang2 = branch2.append(&mut arena, "Swedish");
let lang3 = branch3.append(&mut arena, "Polish");

// Access data by indexing the arena by a token. This operation panics if the
// token is invalid.
assert_eq!(arena[branch3].data, "Slavic");

// Access data by calling "get" on the arena with a token.
assert_eq!(arena.get(lang1).unwrap().data, "English");

// We can also do so mutably (with "get" or through indexing). As you can
// see, calling the "get" functions is more verbose but you get to avoid
// surprise panic attacks (if you don't unwrap like I do here).
arena.get_mut(lang1).unwrap().data = "Icelandic";
assert_eq!(arena[lang1].data, "Icelandic");

// On the flip side, we can access the corresponding token if we already
// have the node
let branch3_node = &arena[branch3];
assert_eq!(branch3, branch3_node.token());
```

We can iterate over the elements by calling iterators on both the tokens
or the nodes. Check the documentation of `Token` or `Node<T>` for a list
of iterators. There is a version of each of the iterators that iterates
over tokens instead of node references. See the docs for details.
```rust
use atree::Tree;

let root_data = "Indo-European";
let (mut arena, root_token) = Tree::with_data(root_data);

// add some more stuff to the tree
let branch1 = root_token.append(&mut arena, "Romance");
let branch2 = root_token.append(&mut arena, "Germanic");
let branch3 = root_token.append(&mut arena, "Slavic");
let lang1 = branch2.append(&mut arena, "English");
let lang2 = branch2.append(&mut arena, "Swedish");
let lang3 = branch3.append(&mut arena, "Polish");

// Getting an iterator from a token and iterate
let children: Vec<_> = root_token.children(&arena).map(|x| x.data).collect();
assert_eq!(&["Romance", "Germanic", "Slavic"], &children[..]);

// Getting an iterator from a node reference (that is if you already have it
// around. To go out of your way to get the node reference before getting
// the iterator seems kind of dumb).
let polish = &arena[lang3];
let ancestors: Vec<_> = polish.ancestors(&arena).map(|x| x.data).collect();
assert_eq!(&["Slavic", "Indo-European"], &ancestors[..]);

// We can also iterate mutably. Unfortunately we can only get mutable
// iterators from the tokens but not from the node references because of
// borrow checking.
for lang in branch2.children_mut(&mut arena) {
    lang.data = "Not romantic enough";
}
assert_eq!(tree[lang1].data, "Not romantic enough");
assert_eq!(tree[lang2].data, "Not romantic enough");
```

To remove a node from the arena, call the `remove` method on the arena. Note
that will also remove all descendants of the node. After removal, the "freed"
memory will be reused if and when new data is inserted.

```rust
use atree::Tree;

let root_data = "Indo-European";
let (mut arena, root_token) = Tree::with_data(root_data);

// add some more stuff to the tree
let branch1 = root_token.append(&mut arena, "Romance");
let branch2 = root_token.append(&mut arena, "Germanic");
let branch3 = root_token.append(&mut arena, "Slavic");
let lang1 = branch2.append(&mut arena, "English");
let lang2 = branch2.append(&mut arena, "Swedish");
let lang3 = branch3.append(&mut arena, "Polish");

assert_eq!(arena.node_count(), 7);
arena.remove(branch2);  // boring languages anyway
assert_eq!(arena.node_count(), 4);
```

## License

MIT
