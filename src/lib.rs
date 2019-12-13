//! An arena based tree structure. Being an arena based tree, this structure is
//! implemented on top of `Vec` and as such eliminates the need for the
//! countless heap allocations or unsafe code that a pointer based tree
//! structure would require. This approach also makes parallel access feasible.
//! On top of the basic node insertion and removal operations, care is taken to
//! provide various convenience functions which enable splitting, merging, and
//! also different kinds of immutable and mutable iterations over the nodes.
//!
//! Most of the code in the crate is `unsafe` free, except for the mutable
//! iterators, where the `unsafe` code is lifted from the core Rust
//! implementation of `IterMut`.
//!
//! # Quick Start
//!
//! The crate consists of three main `struct`s: [`Tree<T>`], [`Token`] and
//! [`Node<T>`]. `Tree<T>` provides the arena in which all data is stored.
//! The data can then be accessed by indexing `Tree<T>` with `Token`. `Node<T>`
//! is a container that encapsulates the data on the tree.
//!
//! We can start by initializing an empty tree and add stuff to it at a later
//! time:
//! ```
//! use itree::Tree;
//!
//! let mut tree = Tree::default();
//! assert!(tree.is_empty());
//!
//! // add stuff to the tree when we feel like it
//! let root_data = "Indo-European";
//! let root_node_token = tree.initialize(root_data);
//! assert_eq!(tree.node_count(), 1)
//! ```
//!
//! Another way is to directly initialize a tree with a root:
//! ```
//! use itree::Tree;
//!
//! let root_data = "Indo-European";
//! let (mut tree, root_token) = Tree::with_root(root_data);
//! assert_eq!(tree.node_count(), 1)
//! ```
//!
//! To add more data to the tree, call the [`append`] method on the tokens (we
//! can't do this directly to the nodes because of the limitations of borrow
//! checking).
//! ```
//! use itree::Tree;
//!
//! let root_data = "Indo-European";
//! let (mut tree, root_token) = Tree::with_root(root_data);
//! root_token.append(&mut tree, "Romance");
//! assert_eq!(tree.node_count(), 2);
//! ```
//!
//! To access/modify existing nodes on the tree, we can use indexing or
//! [`get`]/[`get_mut`].
//! ```
//! use itree::Tree;
//!
//! let root_data = "Indo-European";
//! let (mut tree, root_token) = Tree::with_root(root_data);
//!
//! // add some more stuff to the tree
//! let branch1 = root_token.append(&mut tree, "Romance");
//! let branch2 = root_token.append(&mut tree, "Germanic");
//! let branch3 = root_token.append(&mut tree, "Slavic");
//! let lang1 = branch2.append(&mut tree, "English");
//! let lang2 = branch2.append(&mut tree, "Swedish");
//! let lang3 = branch3.append(&mut tree, "Polish");
//!
//! // Access data by indexing the tree by a token. This operation panics if the
//! // token is invalid.
//! assert_eq!(tree[branch3].data, "Slavic");
//!
//! // Access data by calling "get" on tree with a token.
//! assert_eq!(tree.get(lang1).unwrap().data, "English");
//!
//! // We can also do so mutably (with "get" or through indexing). As you can
//! // see, calling the "get" functions is more verbose but you get to avoid
//! // surprise panic attacks (if you don't unwrap like I do here).
//! tree.get_mut(lang1).unwrap().data = "Icelandic";
//! assert_eq!(tree[lang1].data, "Icelandic");
//!
//! // On the flip side, we can access the corresponding token if we already
//! // have the node
//! let branch3_node = &tree[branch3];
//! assert_eq!(branch3, branch3_node.token());
//! ```
//!
//! We can iterate over the elements by calling iterators on both the tokens
//! or the nodes. Check the documentation of [`Token`] or [`Node<T>`] for a list
//! of iterators. There is a version of each of the iterators that iterates
//! over tokens instead of node references. See the docs for details.
//! ```
//! use itree::Tree;
//!
//! let root_data = "Indo-European";
//! let (mut tree, root_token) = Tree::with_root(root_data);
//!
//! // add some more stuff to the tree
//! let branch1 = root_token.append(&mut tree, "Romance");
//! let branch2 = root_token.append(&mut tree, "Germanic");
//! let branch3 = root_token.append(&mut tree, "Slavic");
//! let lang1 = branch2.append(&mut tree, "English");
//! let lang2 = branch2.append(&mut tree, "Swedish");
//! let lang3 = branch3.append(&mut tree, "Polish");
//!
//! // Getting an iterator from a token and iterate
//! let children: Vec<_> = root_token.children(&tree).map(|x| x.data).collect();
//! assert_eq!(&["Romance", "Germanic", "Slavic"], &children[..]);
//!
//! // Getting an iterator from a node reference (that is if you already have it
//! // around. To go out of your way to get the node reference before getting
//! // the iterator seems kind of dumb).
//! let polish = &tree[lang3];
//! let ancestors: Vec<_> = polish.ancestors(&tree).map(|x| x.data).collect();
//! assert_eq!(&["Slavic", "Indo-European"], &ancestors[..]);
//!
//! // We can also iterate mutably. Unfortunately we can only get mutable
//! // iterators from the tokens but not from the node references because of
//! // borrow checking.
//! for lang in branch2.children_mut(&mut tree) {
//!     lang.data = "Not romantic enough";
//! }
//! assert_eq!(tree[lang1].data, "Not romantic enough");
//! assert_eq!(tree[lang2].data, "Not romantic enough");
//! ```
//!
//! To remove a node, call the [`remove`] method on tree. Note that will also
//! remove all descendants of the node. After removal, the "freed" memory will
//! be reused if and when new data is inserted. There is currently no support
//! for shrinking.
//! ```
//! use itree::Tree;
//!
//! let root_data = "Indo-European";
//! let (mut tree, root_token) = Tree::with_root(root_data);
//!
//! // add some more stuff to the tree
//! let branch1 = root_token.append(&mut tree, "Romance");
//! let branch2 = root_token.append(&mut tree, "Germanic");
//! let branch3 = root_token.append(&mut tree, "Slavic");
//! let lang1 = branch2.append(&mut tree, "English");
//! let lang2 = branch2.append(&mut tree, "Swedish");
//! let lang3 = branch3.append(&mut tree, "Polish");
//!
//! assert_eq!(tree.node_count(), 7);
//! tree.remove(branch2);  // boring languages anyway
//! assert_eq!(tree.node_count(), 4);
//! ```
//!
//! [`Tree<T>`]: struct.Tree.html
//! [`Token`]: struct.Token.html
//! [`Node<T>`]: struct.Node.html
//! [`append`]: struct.Token.html#method.append
//! [`get`]: struct.Tree.html#method.get
//! [`get_mut`]: struct.Tree.html#method.get_mut
//! [`remove`]: struct.Tree.html#method.remove
// TODO: add tree merging capabilities
// TODO: add tree spliting functions
// TODO: shrink to fit
// TODO: use NonZeroUsize instead of usize in Token

mod arena;
pub mod iter;
mod node;
mod token;
mod tree;

pub use token::Token;
pub use tree::Tree;
pub use node::Node;
