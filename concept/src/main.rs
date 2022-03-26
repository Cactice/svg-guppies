#![allow(dead_code)]
#![feature(generic_associated_types)]

use crate::concept_todo::app;
mod concept;
mod concept_todo;

fn main() {
    app();
    println!("Hello, world!");
}
