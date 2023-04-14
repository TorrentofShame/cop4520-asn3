use rand::prelude::*;
use core::time;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

const NUM_THREADS: i32 = 8;
const MINUTES_IN_HOUR: i32 = 60;

fn get_temp() -> i32 {
    rand::thread_rng().gen_range(-100..=70)
}

fn main() {
    let mut cores: Vec<JoinHandle<()>> = Vec::new();
    let temps: Arc<Mutex<Vec<i32>>> = Arc::new(Mutex::new(Vec::new()));

    // Channel so sensors can tell the main thread when a minute has passed.
    let (temp_tx, temp_rx): (Sender<i32>, Receiver<i32>) = mpsc::channel();

    for _t in 0..NUM_THREADS {
        let temps = temps.clone();
        let temp_tx = temp_tx.clone();
        let core = thread::spawn(move || {
            for m in 0..MINUTES_IN_HOUR {
                {
                    let reading = get_temp();
                    let mut tmp_guard = temps.lock().unwrap();
                    tmp_guard.push(reading);
                    //println!("sensor {}: min: {} temp: {}", t, m, reading);
                    if tmp_guard.len() % 8 == 0 {
                        temp_tx.send(m).unwrap();
                    }
                }
                thread::sleep(time::Duration::from_millis(100));
            }
        });
        cores.push(core);
    }

    let mut top_highest: Vec<i32> = Vec::new();
    let mut top_lowest: Vec<i32> = Vec::new();

    let mut int_count = 0;
    for _ in 0..MINUTES_IN_HOUR {
        let _m = temp_rx.recv().unwrap();
        //println!("Received: {}", m);
        if top_highest.len() < 5 {
            let tmps = &temps.lock().unwrap()[(int_count*8)..(8*(int_count+1))];
            top_highest.push(*tmps.iter().max().unwrap());
            top_highest.sort();
        } else {
            let tmps = &temps.lock().unwrap()[(int_count*8)..(8*(int_count+1))];
            let max = *tmps.iter().max().unwrap();
            for (idx, i) in top_highest.clone().iter().enumerate() {
                if max > *i {
                    top_highest[idx] = max;
                    break;
                }
            }
        }

        if top_lowest.len() < 5 {
            let tmps = &temps.lock().unwrap()[(int_count*8)..(8*(int_count+1))];
            top_lowest.push(*tmps.iter().min().unwrap());
            top_lowest.sort();
            top_lowest.reverse();
        } else {
            let tmps = &temps.lock().unwrap()[(int_count*8)..(8*(int_count+1))];
            let min = *tmps.iter().min().unwrap();
            for (idx, i) in top_lowest.clone().iter().enumerate() {
                if min < *i {
                    top_lowest[idx] = min;
                    break;
                }
            }
        }

        int_count += 1;
    }

    // Wait for shutdown
    for core in cores.into_iter() {
        core.join().unwrap();
    }

    let mut largest_interval = 0;
    let mut largest_change = 0;

    // Get the average temperature for each minute.
    let minute_averages: Vec<i32> = temps.lock().unwrap().chunks(8).into_iter().map(|c| -> i32 { c.iter().sum() }).collect();
    for i in 0..6 {
        let d_temp = minute_averages[i].abs_diff(minute_averages[i+1]);
        if d_temp > largest_change {
            largest_change = d_temp;
            largest_interval = i;
        }
    }

    println!("Hourly Report");
    println!("==========");
    println!("Top 5 highest temperatures: {:?}", top_highest);
    println!("Top 5 lowest temperatures: {:?}", top_lowest);
    println!("10-minute interval with the highest temperature difference: {} with a change of {}F",
        format!("{} minute to {} minute", largest_interval * 10, (largest_interval + 1) * 10),
        largest_change);
    println!("==========");

}
