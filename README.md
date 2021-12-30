# tree-sitter-traversal
Traversal of tree-sitter Trees and any arbitrary tree with a TreeCursor-like interface.

[![build_status](https://github.com/skmendez/tree-sitter-traversal/actions/workflows/rust.yml/badge.svg)](https://github.com/skmendez/tree-sitter-traversal/actions)
[![Documentation](https://docs.rs/tree-sitter-traversal/badge.svg)](https://docs.rs/tree-sitter-traversal)
[![crates.io](https://img.shields.io/crates/v/tree-sitter-traversal.svg)](https://crates.io/crates/tree-sitter-traversal)

Using cursors, iteration over the tree can be implemented in a very efficient manner which performs no additional heap allocation beyond what might be required by the Cursor. The state required for pre-order and post-order traversal is very minimal; for pre-order traversal, all that is required is to maintain whether the traversal is complete, and for post-order traversal, we also maintain if the cursor is currently traversing up or down the tree.

### Usage

Add this to your `Cargo.toml`

```toml
[dependencies]
tree-sitter-traversal = "0.1.2"
```

### Example

```rust
use tree_sitter::{Node, Tree};

use tree_sitter_traversal::{traverse, traverse_tree, Order};
fn get_tree() -> Tree {
    use tree_sitter::Parser;
    let mut parser = Parser::new();
    let lang = tree_sitter_javascript::language();
    parser.set_language(lang).expect("Error loading JavaScript grammar");
    return parser.parse("function(x) { return x * 2; }", None).expect("Error parsing provided code");
}

fn main() {
    use std::collections::HashSet;
    use std::iter::FromIterator;
    let tree: Tree = get_tree();
    let preorder: Vec<Node<'_>> = traverse(tree.walk(), Order::Pre).collect::<Vec<_>>();
    let postorder: Vec<Node<'_>> = traverse_tree(&tree, Order::Post).collect::<Vec<_>>();
    // For any tree with more than just a root node,
    // the order of preorder and postorder will be different
    assert_ne!(preorder, postorder);
    // However, they will have the same amount of nodes
    assert_eq!(preorder.len(), postorder.len());
    // Specifically, they will have the exact same nodes, just in a different order
    assert_eq!(
        <HashSet<_>>::from_iter(preorder.into_iter()),
        <HashSet<_>>::from_iter(postorder.into_iter())
    );   
}
```

### Features

Though this library was designed to be used for `tree-sitter`, that usage is optional, as it can also be used by any struct which implements the `Cursor` trait. When the `tree-sitter` feature is disabled, the library is actually `#![no_std]`. To use without `tree-sitter`, add this to your Cargo.toml instead:

```toml
[dependencies.tree-sitter-traversal]
version = "0.1.2"
default-features = false
```