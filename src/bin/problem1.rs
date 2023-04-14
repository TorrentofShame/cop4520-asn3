use rand::prelude::*;
use std::collections::VecDeque;
use std::sync::atomic::AtomicUsize;
use std::sync::{atomic::Ordering, Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

const NUM_OF_SERVENTS: i32 = 4;
const NUM_OF_GUESTS: i32 = 500000;

struct Node {
    tag: i32,
    next: Option<Arc<Mutex<Node>>>,
}

impl Node {
    fn new(v: i32) -> Self {
        Node {
            tag: v,
            next: None
        }
    }
}

struct Chain {
    head: Option<Arc<Mutex<Node>>>,
}

impl Default for Chain {
    fn default() -> Self {
        Chain {
            head: None,
        }
    }
}

impl Chain {
    pub fn insert(&mut self, v: i32) {
        let new_v = Arc::new(Mutex::new(Node::new(v)));

        if let None = self.head {
            self.head = Some(new_v.clone());
            return;
        }

        let mut prev: Option<Arc<Mutex<Node>>> = None;
        let mut cur = self.head.clone();

        while let Some(node) = cur {
            let cur_guard = node.lock().unwrap();

            if v < cur_guard.tag {
                if let Some(p) = prev {
                    let mut p_guard = p.lock().unwrap();
                    p_guard.next = Some(new_v.clone());
                } else {
                    self.head = Some(new_v.clone());
                }

                new_v.lock().unwrap().next = Some(node.clone());
                return;
            }

            prev = Some(node.clone());
            cur = cur_guard.next.clone();
        }

        prev.unwrap().lock().unwrap().next = Some(new_v);
    }

    pub fn contains(&self, v: &i32) -> bool {
        let mut cur = self.head.clone();

        while let Some(node) = cur {
            let n_guard = node.lock().unwrap();

            if &n_guard.tag == v {
                return true
            }

            cur = n_guard.next.clone();
        }

        return false
    }

    /// Pops node from top of the chain 
    pub fn pop(&mut self) -> Option<i32> {
        self.head.take().map(|head| {
            let head = head.lock().unwrap();
            let next = head.next.clone();
            self.head = next;
            head.tag
        })
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

            // There is a 25% chance that the Minotaur will ask for a servent to find a tag.
            let should_find_tag: bool = rand::thread_rng().gen_ratio(1,4);

            loop {
                if should_find_tag && chain.lock().unwrap().head.is_some() {
                    // Minotaur wants a random tag found.
                    let to_find = rand::thread_rng().gen_range(0..NUM_OF_GUESTS);
                    if chain.lock().unwrap().contains(&to_find) {
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
                            if let None = chain.lock().unwrap().head {
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
