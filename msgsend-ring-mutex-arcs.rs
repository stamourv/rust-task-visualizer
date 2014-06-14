// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// This test creates a bunch of tasks that simultaneously send to each
// other in a ring. The messages should all be basically
// independent.
// This is like msgsend-ring-pipes but adapted to use Arcs.

// This also serves as a pipes test, because Arcs are implemented with pipes.

#![feature(phase)]

#[phase(plugin, link)]
extern crate green;
extern crate rtinstrument;
extern crate time;

use std::sync::{Arc, Future, Mutex};
use std::os;

// A poor man's pipe.
type Pipe = Arc<Mutex<Vec<uint>>>;

fn send(p: &Pipe, msg: uint) {
    let mut arr = p.lock();
    arr.push(msg);
    arr.cond.signal();
}
fn recv(p: &Pipe) -> uint {
    let mut arr = p.lock();
    while arr.is_empty() {
        arr.cond.wait();
    }
    arr.pop().unwrap()
}

fn init() -> (Pipe,Pipe) {
    let m = Arc::new(Mutex::new(Vec::new()));
    ((&m).clone(), m)
}


fn thread_ring(i: uint, count: uint, num_chan: Pipe, num_port: Pipe) {
    // Send/Receive lots of messages.
    for j in range(0u, count) {
        send(&num_chan, i * j);
        let _n = recv(&num_port);
    };
}

fn main() {
    let args = os::args();
    let args = if os::getenv("RUST_BENCH").is_some() {
        vec!("".to_string(), "100".to_string(), "10000".to_string())
    } else if args.len() <= 1u {
        vec!("".to_string(), "10".to_string(), "100".to_string())
    } else {
        args.clone().move_iter().collect()
    };

    let num_tasks = from_str::<uint>(args.get(1).as_slice()).unwrap();
    let msg_per_task = from_str::<uint>(args.get(2).as_slice()).unwrap();

    let (mut num_chan, num_port) = init();

    let start = time::precise_time_s();

    // create the ring
    let mut futures = Vec::new();

    for i in range(1u, num_tasks) {
        //println!("spawning %?", i);
        let (new_chan, num_port) = init();
        let num_chan_2 = num_chan.clone();
        let new_future = Future::spawn(proc() {
            thread_ring(i, msg_per_task, num_chan_2, num_port)
        });
        futures.push(new_future);
        num_chan = new_chan;
    };

    // do our iteration
    thread_ring(0, msg_per_task, num_chan, num_port);

    // synchronize
    for f in futures.mut_iter() {
        f.get()
    }

    let stop = time::precise_time_s();

    // all done, report stats.
    let num_msgs = num_tasks * msg_per_task;
    let elapsed = stop - start;
    let rate = (num_msgs as f64) / elapsed;

    println!("Sent {} messages in {} seconds", num_msgs, elapsed);
    println!("  {} messages / second", rate);
    println!("  {} μs / message", 1000000. / rate);
}

green_start!(real_main)

fn real_main() {
    let msgs = rtinstrument::instrument::<green::task::GreenTask>(main);

    for msg in msgs.iter() {
        println!("{}", msg);
    }
}