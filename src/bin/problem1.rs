use rand::prelude::*;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicPtr, AtomicUsize};
use std::sync::{atomic::Ordering, Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::ptr;

const NUM_OF_SERVENTS: i32 = 4;
const NUM_OF_GUESTS: i32 = 500000;

struct Node {
    tag: i32,
    next: AtomicPtr<Mutex<Node>>,
}

impl Node {
    fn new(v: i32) -> Self {
        Node {
            tag: v,
            next: AtomicPtr::default(),
        }
    }
}

struct Chain {
    head: AtomicPtr<Mutex<Node>>,
}

impl Default for Chain {
    fn default() -> Self {
        Chain {
            head: AtomicPtr::default(),
        }
    }
}

impl Chain {
    pub fn insert(&self, v: i32) {
        let mut prev_ptr: *mut Mutex<Node> = ptr::null_mut();
        let mut cur_ptr = self.head.load(Ordering::SeqCst);
        while !cur_ptr.is_null() {
            let cur_ref = unsafe { cur_ptr.as_ref().unwrap() }.lock().unwrap();
            // If v < the current ref tag, then we place v before the current ref
            if v < cur_ref.tag {
                let new_v = Node::new(v);
                new_v.next.store(cur_ptr, Ordering::SeqCst);

                if prev_ptr.is_null() { // v becomes new head
                    self.head.store(&mut Mutex::new(new_v), Ordering::SeqCst);
                    return;
                }
                let prev_ref = unsafe { prev_ptr.as_ref().unwrap() }.lock().unwrap();
                prev_ref.next.store(&mut Mutex::new(new_v), Ordering::SeqCst);
            }
            // If not, set cur_ptr to the next in the linked list
            prev_ptr = cur_ptr;
            cur_ptr = cur_ref.next.load(Ordering::SeqCst);
        }
        if cur_ptr.is_null() {
            let new_v = Node::new(v);
            self.head.store(&mut Mutex::new(new_v), Ordering::SeqCst);
        }
    }

    pub fn contains(&self, v: i32) -> bool {
        let mut cur_ptr = self.head.load(Ordering::SeqCst);
        while !cur_ptr.is_null() {
            let cur_ref = unsafe { cur_ptr.as_ref().unwrap() }.lock().unwrap();
            // If the current ref tag is what we're looking for, return true
            if cur_ref.tag == v {
                return true
            }
            // If not, set cur_ptr to the next in the linked list
            cur_ptr = cur_ref.next.load(Ordering::SeqCst);
        }
        return false
    }

    /// Pops node from top of the chain 
    pub fn pop(&self) -> Option<i32> {
        let mut head_ptr = self.head.load(Ordering::SeqCst);
        if !head_ptr.is_null() {
            let head_ref = unsafe { head_ptr.as_ref().unwrap() }.lock().unwrap();
            let tag = head_ref.tag;

            assert!(tag > 0, "Tag should not be negative! {}", tag);
            assert!(tag < NUM_OF_GUESTS, "Tag cannot be bigger than num of guests {}", tag);

            // Get the Current Head's next node, which will be the new head node
            let next_ptr = head_ref.next.load(Ordering::SeqCst);
            if !next_ptr.is_null() {

                // Set head to the next head
                loop {
                    // Deal with if the head has changed since we started.
                    match self.head.compare_exchange(head_ptr, next_ptr, Ordering::SeqCst, Ordering::SeqCst) {
                        Ok(_) => { break; },
                        Err(p) => {
                            let old_head_ptr = head_ptr;
                            head_ptr = p;
                            if !head_ptr.is_null() {
                                let phead_ref = unsafe { head_ptr.as_ref().unwrap() }.lock().unwrap();

                                // phead_ref.next might be != to the old head so bugs might be here
                                //phead_ref.next.store(next_ptr, Ordering::SeqCst);
                                match phead_ref.next.compare_exchange(old_head_ptr, next_ptr, Ordering::SeqCst, Ordering::SeqCst) {
                                    Ok(_) => {},
                                    Err(j) => {
                                        if j.is_null() {
                                            panic!("phead_ref.next is not old_head and is null!");
                                        } else {
                                            panic!("phead_ref.next is not old_head nor is it null!");
                                        }
                                    }
                                }
                            }
                        }
                    };
                }
            } else {
                self.head.store(ptr::null_mut(), Ordering::SeqCst);
            }

            // Drop the old head pointer
            drop(head_ptr);

            Some(tag)
        } else { None }
    }
}

fn get_presents() -> VecDeque<i32> {
    let mut rng = thread_rng();
    let mut tmp: Vec<i32> = (0..NUM_OF_GUESTS).collect();
    tmp.shuffle(&mut rng);

    VecDeque::from(tmp)
}

// Action 3 (find tag in chain) is not here as that is randomly selected
// and is not alternated to and from, in the case of the other actions.
#[derive(Debug)]
enum ServentAction {
    AddPresent,
    ThankYou
}
// TODO: See if randomly choosing a starting actions is good

impl Iterator for ServentAction {
    type Item = ServentAction;

    // Alternate between actions
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::AddPresent => Some(Self::ThankYou),
            Self::ThankYou => Some(Self::AddPresent)
        }
    }
}

fn main() {
    let mut servents: Vec<JoinHandle<()>> = Vec::new();
    let bag = Arc::new(Mutex::new(get_presents()));
    let chain = Arc::new(Mutex::new(Chain::default()));
    let note_counter = Arc::new(AtomicUsize::new(0));

    for i in 0..NUM_OF_SERVENTS {
        let bag = bag.clone();
        let chain = chain.clone();
        let note_counter = note_counter.clone();
        let servent = thread::spawn(move || {
            let mut action = ServentAction::AddPresent;

            let should_find_tag: bool = false;//rand::random();

            loop {
                if should_find_tag {
                    // Minotaur wants a random tag found.
                    let to_find = rand::thread_rng().gen_range(0..NUM_OF_GUESTS);
                    if chain.lock().unwrap().contains(to_find) {
                        println!("Minotaur asked servent {} to find present {}...The present was found.", i, to_find);
                    } else {
                        println!("Minotaur asked servent {} to find present {}...The present was not found.", i, to_find);
                    }
                    continue;
                }

                match action {
                    ServentAction::AddPresent => {
                        // Grab the present from the top of the bag
                        let tag = bag.lock().unwrap().pop_front();
                        if let Some(t) = tag {
                            chain.lock().unwrap().insert(t);
                            println!("Servent {} added gift {} to the chain.", i, t);
                        } else {
                            if chain.lock().unwrap().head.load(Ordering::SeqCst).is_null() {
                                println!("Servent {} is finished!", i);
                                break;
                            }
                        }
                    },
                    ServentAction::ThankYou => {
                        let present = chain.lock().unwrap().pop();
                        if let Some(p) = present {
                            println!("Servent {} wrote thank you card for gift {}", i, p);
                            note_counter.fetch_add(1, Ordering::Relaxed);
                        } else {
                            if bag.lock().unwrap().is_empty() {
                                println!("Servent {} is finished!", i);
                                break;
                            }
                        }
                    }
                };

                // Go to the next action
                action = action.next().unwrap();
            }
        });
        servents.push(servent);
    }

    // Wait for shutdown
    for servent in servents.into_iter() {
        servent.join().unwrap();
    }

    println!("The servents finished! {} thank you letters have been written!", note_counter.load(Ordering::SeqCst));
}
