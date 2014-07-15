#![feature(phase)]

#[phase(plugin, link)]
extern crate green;
extern crate rtinstrument;

green_start!(main)

fn main() {
    let msgs = rtinstrument::instrument(work);

    for msg in msgs.iter() {
        println!("{}", msg);
    }
}

fn work() {
    let (tx, rx) = channel();
    for _ in range(0u, 10) {
        let tx = tx.clone();
        spawn(proc() {
            println!("baz");
            tx.send(());
        });
    }
    for _ in range(0u, 10) {
        rx.recv();
    }
}
