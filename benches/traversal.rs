#![feature(test)]
extern crate test;

use test::Bencher;

use tree_sitter_traversal::{traverse_tree, Order};
use tree_sitter::{Parser, Tree};
use std::fs::File;
use std::io::Read;
use std::path::Path;

fn dogfood() -> Tree {
    let mut file = File::open("./src/lib.rs").expect("src/lib.rs should exist");
    let mut code = String::new();
    file.read_to_string(&mut code).unwrap();

    let mut parser = Parser::new();
    let lang = tree_sitter_javascript::language();
    parser
        .set_language(lang)
        .expect("Error loading JavaScript grammar");
    return parser
        .parse(code, None)
        .expect("Error parsing provided code");
}

#[bench]
fn dogfooding_preorder(b: &mut Bencher) {
   let tree = dogfood();

    b.iter(|| {
        traverse_tree(&tree, Order::Pre).collect::<Vec<_>>()
    });
}

#[bench]
fn dogfooding_postorder(b: &mut Bencher) {
    let tree = dogfood();

    b.iter(|| {
        traverse_tree(&tree, Order::Post).collect::<Vec<_>>()
    });
}