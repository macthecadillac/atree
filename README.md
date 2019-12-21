# atree

## Arena based tree structure with node removal support

[![Build Status](https://travis-ci.com/macthecadillac/atree.svg?branch=master)](https://travis-ci.com/macthecadillac/atree)
[![Crates.io](https://img.shields.io/crates/v/atree.svg)](https://crates.io/crates/atree)

An arena based tree structure, backed by a custom allocator (ultimately
built on `Vec`) that makes node removal a possibility. On top of the basic
node insertion and removal operations, there are also many kinds of
immutable and mutable iterators provided for various kinds of tree traversal
operations.

Most of the code in the crate is `unsafe` free, except for the mutable
iterators, where the `unsafe` code is lifted from the core Rust
implementation of `IterMut`.

## General Guide to the API

The crate consists of three main `struct`s: [`Arena<T>`], [`Token`] and
[`Node<T>`]. `Arena<T>` provides the arena in which all data is stored.
The data can then be accessed by indexing `Arena<T>` with `Token`. `Node<T>`
is a container that encapsulates the data on the tree.

As a general rule of thumb, methods that affect the memory layout such as
splitting and merging arenas, or methods to create and destroy nodes regardless
of existing tree structures like creating a free node, are defined on
`Arena<T>`. Methods that alter pre-existing tree structures such as adding
nodes with respect to existing ones ([`append`] or [`insert_after`] for
instance) or splitting and attaching existing trees are defined on `Tokens`.

When it comes to iterating, iterators can be created from methods on both
`Token` and `Node<T>`. There are two versions of iterators, iterators over
tokens or references to the nodes. Both can be created by methods on `Token`
and `Node<T>`. However, due to the rules of borrow checking, mutable
iterators over the node references are only defined on `Token`.

## Crate Feature Flags
  - `serde`: support for serde 1.x. Optional feature/dependency.

## Usage Examples

We can start by initializing an empty arena and add stuff to it at a later
time:
```rust
use atree::Arena;

let mut arena = Arena::default();
assert!(arena.is_empty());

// create a tree in the arena
let data = "Indo-European";
let token = arena.new_node(data);
assert_eq!(arena.node_count(), 1)
```

There is a shortcut to the above operation:
```rust
use atree::Arena;

let data = "Indo-European";
let (mut arena, token) = Arena::with_data(data);
assert_eq!(arena.node_count(), 1)
```

To add more data to the tree, call the [`append`] method on the tokens (we
can't do this directly to the nodes because of the limitations of borrow
checking).
```rust
use atree::Arena;

let root_data = "Indo-European";
let (mut arena, root_token) = Arena::with_data(root_data);
root_token.append(&mut arena, "Romance");
assert_eq!(arena.node_count(), 2);
```

To access/modify existing nodes in the tree, we can use indexing or
[`get`]/[`get_mut`].
```rust
use atree::Arena;

let root_data = "Indo-European";
let (mut arena, root_token) = Arena::with_data(root_data);

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
or the nodes. Check the documentation of [`Token`] or [`Node<T>`] for a list
of iterators. There is a version of each of the iterators that iterates
over tokens instead of node references. See the docs for details.
```rust
use atree::Arena;

let root_data = "Indo-European";
let (mut arena, root_token) = Arena::with_data(root_data);

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
assert_eq!(arena[lang1].data, "Not romantic enough");
assert_eq!(arena[lang2].data, "Not romantic enough");
```

To remove a single node from the arena, use the [`remove`] method. This will
detach all the children of the node from the tree (but not remove them from
memory).
```rust
use atree::Arena;
use atree::iter::TraversalOrder;

// root node that we will attach subtrees to
let root_data = "Indo-European";
let (mut arena, root) = Arena::with_data(root_data);

// the Germanic branch
let germanic = root.append(&mut arena, "Germanic");
let west = germanic.append(&mut arena, "West");
let scots = west.append(&mut arena, "Scots");
let english = west.append(&mut arena, "English");

// detach the west branch from the main tree
let west_children = arena.remove(west);

// the west branch is gone from the original tree
let mut iter = root.subtree(&arena, TraversalOrder::Pre)
    .map(|x| x.data);
assert_eq!(iter.next(), Some("Indo-European"));
assert_eq!(iter.next(), Some("Germanic"));
assert!(iter.next().is_none());

// its children are still areound
let mut iter = west_children.iter().map(|&t| arena[t].data);
assert_eq!(iter.next(), Some("Scots"));
assert_eq!(iter.next(), Some("English"));
assert!(iter.next().is_none());
```

To uproot a tree from the arena, call the [`uproot`] method on the arena.
Note that will also remove all descendants of the node. After removal, the
"freed" memory will be reused if and when new data is inserted.
```rust
use atree::Arena;

let root_data = "Indo-European";
let (mut arena, root_token) = Arena::with_data(root_data);

// add some more stuff to the tree
let branch1 = root_token.append(&mut arena, "Romance");
let branch2 = root_token.append(&mut arena, "Germanic");
let branch3 = root_token.append(&mut arena, "Slavic");
let lang1 = branch2.append(&mut arena, "English");
let lang2 = branch2.append(&mut arena, "Swedish");
let lang3 = branch3.append(&mut arena, "Polish");

assert_eq!(arena.node_count(), 7);
arena.uproot(branch2);  // boring languages anyway
assert_eq!(arena.node_count(), 4);
```

[`Arena<T>`]: struct.Arena.html
[`Token`]: struct.Token.html
[`Node<T>`]: struct.Node.html
[`append`]: struct.Token.html#method.append
[`insert_after`]: struct.Token.html#method.insert_after
[`get`]: struct.Arena.html#method.get
[`get_mut`]: struct.Arena.html#method.get_mut
[`uproot`]: struct.Arena.html#method.uproot
[`remove`]: struct.Arena.html#method.remove

## License: MIT
