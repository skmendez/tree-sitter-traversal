use tree_sitter::{Parser, TreeCursor, Node, Tree};

/// Order to iterate through the tree; for n-ary trees only
/// Pre-order and Post-order make sense
#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone)]
pub enum Order {
    Pre,
    Post
}

/// Iterative traversal of the tree; serves as a reference for both
/// PreorderTraversal and PostorderTraversal, as they both will call the exact same
/// cursor methods in the exact same order as this function for a given tree; the order
/// is also the same as traverse_recursive.
fn traverse_iterative<'a, F>(tree: &'a Tree, order: Order, mut cb: F) where F: FnMut(Node<'a>) {
    let mut c = tree.walk();
    loop {
        // This is the first time we've encountered the node, so we'll call if preorder
        if order == Order::Pre { cb(c.node()); }

        // Keep travelling down the tree as far as we can
        if c.goto_first_child() {
            continue;
        }

        let node = c.node();

        // If we can't travel any further down, try going to next sibling and repeating
        if c.goto_next_sibling() {
            // If we succeed in going to the previous nodes sibling,
            // we won't be encountering that node again, so we'll call if postorder
            if order == Order::Post { cb(node); }
            continue;
        }

        // Otherwise, we must travel back up; we'll loop until we reach the root or can
        // go to the next sibling of a node again.
        loop {
            // Since we're retracing back up the tree, this is the last time we'll encounter
            // this node, so we'll call if postorder
            if order == Order::Post { cb(c.node()); }
            if !c.goto_parent() {
                // We have arrived back at the root, so we are done.
                return;
            }

            let node = c.node();

            if c.goto_next_sibling() {
                // If we succeed in going to the previous node's sibling,
                // we will go back to travelling down that sibling's tree, and we also
                // won't be encountering the previous node again, so we'll call if postorder
                if order == Order::Post { cb(node); }
                break;
            }
        }
    }
}

/// Idiomatic recursive traversal of the tree; this version is easier to understand
/// conceptually, but the recursion is actually unnecessary and can cause stack overflow.
fn traverse_recursive<'a, F>(tree: &'a Tree, order: Order, mut cb: F) where F: FnMut(Node<'a>) {
    traverse_helper(&mut tree.walk(), order, &mut cb);
}

fn traverse_helper<'a, F>(c: &mut TreeCursor<'a>, order: Order, cb: &mut F) where F: FnMut(Node<'a>) {
    // If preorder, call the callback when we first touch the node
    if order == Order::Pre {
        cb(c.node());
    }
    if c.goto_first_child() {
        // If there is a child, recursively call on
        // that child and all its siblings
        loop {
            traverse_helper(c, order, cb);
            if !c.goto_next_sibling() {
                break;
            }
        }
        // Make sure to reset back to the original node;
        // this must always return true, as we only get here if we go to a child
        // of the original node.
        assert!(c.goto_parent());
    }
    // If preorder, call the callback after the recursive calls on child nodes
    if order == Order::Post {
        cb(c.node());
    }
}

struct PreorderTraverse<'a> {
    cursor: Option<TreeCursor<'a>>,
}

impl<'a> PreorderTraverse<'a> {
    pub fn new(tree: &'a Tree) -> Self {
        PreorderTraverse { cursor: Some(tree.walk()) }
    }
}

impl<'a> Iterator for PreorderTraverse<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let c = match self.cursor.as_mut() {
            None => {return None;}
            Some(c) => c
        };

        // We will always return the node we were on at the start;
        // the node we traverse to will either be returned on the next iteration,
        // or will be back to the root node, at which point we'll clear out
        // the reference to the cursor
        let node = c.node();

        // First, try to go to a child or a sibling; if either succeed, this will be the
        // first time we touch that node, so it'll be the next starting node
        if c.goto_first_child() || c.goto_next_sibling() {
            return Some(node);
        }

        loop {
            // If we can't go to the parent, then that means we've reached the root, and our
            // iterator will be done in the next iteration
            if !c.goto_parent() {
                self.cursor = None;
                break;
            }

            // If we get to a sibling, then this will be the first time we touch that node,
            // so it'll be the next starting node
            if c.goto_next_sibling() {
                break;
            }
        }

        return Some(node);
    }
}


struct PostorderTraverse<'a> {
    cursor: Option<TreeCursor<'a>>,
    retracing: bool
}

impl<'a> PostorderTraverse<'a> {
    pub fn new(tree: &'a Tree) -> Self {
        PostorderTraverse {
            cursor: Some(tree.walk()),
            retracing: false
        }
    }
}

impl<'a> Iterator for PostorderTraverse<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let c = match self.cursor.as_mut() {
            None => {return None;}
            Some(c) => c
        };

        // For the postorder traversal, we will only return a node when we are travelling back up
        // the tree structure. Therefore, we go all the way to the leaves of the tree immediately,
        // and only when we are retracing do we return elements
        if !self.retracing {
            while c.goto_first_child() {}
        }

        // Much like in preorder traversal, we want to return the node we were previously at.
        // We know this will be the last time we touch this node, as we will either be going
        // to its next sibling or retracing back up the tree
        let node = c.node();
        if c.goto_next_sibling() {
            // If we successfully go to a sibling of this node, we want to go back down
            // the tree on the next iteration
            self.retracing = false;
        } else {
            // If we weren't already retracing, we are now; travel upwards until we can
            // go to the next sibling or reach the root again
            self.retracing = true;
            if !c.goto_parent() {
                // We've reached the root again, and our iteration is done
                self.cursor = None;
            }
        }

        return Some(node);
    }
}

struct Traverse<'a> {
    inner: TraverseInner<'a>
}

enum TraverseInner<'a> {
    Post(PostorderTraverse<'a>),
    Pre(PreorderTraverse<'a>)
}

impl<'a> Traverse<'a> {
    pub fn new(tree: &'a Tree, order: Order) -> Self {
        let inner = match order {
            Order::Pre => TraverseInner::Pre(PreorderTraverse::new(tree)),
            Order::Post => TraverseInner::Post(PostorderTraverse::new(tree))
        };
        Self { inner }
    }
}

pub fn traverse_iter(tree: &Tree, order: Order) -> impl Iterator<Item=Node> {
    return Traverse::new(tree, order);
}

impl<'a> Iterator for Traverse<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner {
            TraverseInner::Post(ref mut i) => i.next(),
            TraverseInner::Pre(ref mut i) => i.next()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_tree() -> Tree {
        let code =
r#"
const DOUBLE = 2;

function double(x, y, z=123) {
    return DOUBLE * x;
}"#;
        let mut parser = Parser::new();
        let lang = tree_sitter_javascript::language();
        parser.set_language(lang).expect("Error loading JavaScript grammar");
        return parser.parse(code, None).unwrap();
    }

    #[test]
    fn repeat() {
        let parsed = get_tree();
        traverse_recursive(&parsed, Order::Post, |node| {eprintln!("{:?}", node)});
    }

    #[test]
    fn preorder_eq() {
        let parsed = get_tree();
        let mut v = Vec::new();
        traverse_recursive(&parsed, Order::Pre, |node| {v.push(node)});
        let mut e = Vec::new();
        traverse_iterative(&parsed, Order::Pre, |node| {e.push(node)});
        assert_eq!(e, v);
        eprintln!("{:?}", e);
    }

    #[test]
    fn new_eq() {
        let parsed = get_tree();
        let t = PreorderTraverse::new(&parsed);
        let v = t.collect::<Vec<_>>();
        let mut e = Vec::new();
        eprintln!("{:?}", parsed.root_node().to_sexp());
        traverse_recursive(&parsed, Order::Pre, |node| {e.push(node)});
        assert_eq!(v, e);
        eprintln!("{:?}", v);
        "(program (function_declaration name: (identifier) parameters: (formal_parameters (identifier)) body: (statement_block)))";
    }

    #[test]
    fn postorder_eq() {
        let parsed = get_tree();
        let t = traverse_iter(&parsed, Order::Post);
        let v = t.collect::<Vec<_>>();
        let mut e = Vec::new();
        eprintln!("{:?}", parsed.root_node().to_sexp());
        traverse_iterative(&parsed, Order::Post, |node| {e.push(node)});
        assert_eq!(v, e);
        eprintln!("{:?}", v);
        "(program (function_declaration name: (identifier) parameters: (formal_parameters (identifier)) body: (statement_block)))";
    }
}