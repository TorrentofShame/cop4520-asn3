#![allow(dead_code, unused_imports)]
use rand::prelude::*;
use std::sync::atomic::{AtomicPtr, AtomicI32};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{atomic::AtomicBool, atomic::Ordering, mpsc, Arc};
use std::thread;
use std::thread::JoinHandle;

const NUM_OF_SERVENTS: i32 = 4;
const NUM_OF_GUESTS: i32 = 500_000;

// Implement Concurrent Linked-List

struct Node {
    value: AtomicI32,
    next: AtomicPtr<Node>,
    prev: AtomicPtr<Node>,
}

impl Node {
    fn new(v: i32) -> Self {
        Node {
            value: AtomicI32::new(v),
            next: AtomicPtr::default(),
            prev: AtomicPtr::default(),
        }
    }
}

struct ConcurrentLinkedList {
    head: AtomicPtr<Node>,
}

impl Default for ConcurrentLinkedList {
    fn default() -> Self {
        ConcurrentLinkedList {
            head: AtomicPtr::default(),
        }
    }
}

impl ConcurrentLinkedList {
    pub fn insert(&self, v: i32) {
        todo!()
    }

    pub fn get(&self, v: i32) {
        todo!()
    }

    pub fn remove(&self, v: i32) {
        todo!()
    }
}

fn get_presents() -> Vec<i32> {
    let mut rng = thread_rng();
    let mut tmp: Vec<i32> = (0..NUM_OF_GUESTS).collect();
    tmp.shuffle(&mut rng);
    return tmp;
}

fn main() {
    let mut servents: Vec<JoinHandle<()>> = Vec::new();
    #[allow(unused_variables)]
    let bag = get_presents();

    #[allow(unused_variables)]
    for i in 0..NUM_OF_SERVENTS {
        servents.push(thread::spawn(|| {
        }));
    }

    // Wait for shutdown
    for servent in servents.into_iter() {
        servent.join().unwrap();
    }
}
