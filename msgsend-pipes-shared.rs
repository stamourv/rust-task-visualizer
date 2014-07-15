// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// A port of the simplistic benchmark from
//
//    http://github.com/PaulKeeble/ScalaVErlangAgents
//
// I *think* it's the same, more or less.

// This version uses pipes with a shared send endpoint. It should have
// different scalability characteristics compared to the select
// version.

#![feature(phase)]

#[phase(plugin, link)]
extern crate green;
extern crate rtinstrument;
extern crate time;
extern crate debug;

use std::os;
use std::task;
use std::task::TaskBuilder;

fn move_out<T>(_x: T) {}

enum Request {
    GetCount,
    Bytes(uint),
    Stop
}

fn server(requests: &Receiver<Request>, responses: &Sender<uint>) {
    let mut count = 0u;
    let mut done = false;
    while !done {
        match requests.recv_opt() {
          Ok(GetCount) => { responses.send(count.clone()); }
          Ok(Bytes(b)) => {
            //println!("server: received {:?} bytes", b);
            count += b;
          }
          Err(..) => { done = true; }
          _ => { }
        }
    }
    responses.send(count);
    //println!("server exiting");
}

fn run(args: &[String]) {
    let (to_parent, from_child) = channel();
    let (to_child, from_parent) = channel();

    let size = from_str::<uint>(args[1].as_slice()).unwrap();
    let workers = from_str::<uint>(args[2].as_slice()).unwrap();
    let num_bytes = 100;
    let start = time::precise_time_s();
    let mut worker_results = Vec::new();
    for _ in range(0u, workers) {
        let to_child = to_child.clone();
        worker_results.push(TaskBuilder::new().try_future(proc() {
            for _ in range(0u, size / workers) {
                //println!("worker {:?}: sending {:?} bytes", i, num_bytes);
                to_child.send(Bytes(num_bytes));
            }
            //println!("worker {:?} exiting", i);
        }));
    }
    task::spawn(proc() {
        server(&from_parent, &to_parent);
    });

    for r in worker_results.move_iter() {
        r.unwrap().ok().unwrap();
    }

    //println!("sending stop message");
    to_child.send(Stop);
    move_out(to_child);
    let result = from_child.recv();
    let end = time::precise_time_s();
    let elapsed = end - start;
    print!("Count is {:?}\n", result);
    print!("Test took {:?} seconds\n", elapsed);
    let thruput = ((size / workers * workers) as f64) / (elapsed as f64);
    print!("Throughput={} per sec\n", thruput);
    assert_eq!(result, num_bytes * size);
}

fn main() {
    let args = os::args();
    let args = if os::getenv("RUST_BENCH").is_some() {
        vec!("".to_string(), "1000000".to_string(), "10000".to_string())
    } else if args.len() <= 1u {
        vec!("".to_string(), "10000".to_string(), "4".to_string())
    } else {
        args.move_iter().map(|x| x.to_string()).collect()
    };

    println!("{}", args);
    run(args.as_slice());
}

green_start!(real_main)

fn real_main() {
    let msgs = rtinstrument::instrument(main);

    for msg in msgs.iter() {
        println!("{}", msg);
    }
}
