#![feature(phase)]

#[phase(plugin, link)]
extern crate green;
extern crate rtinstrument;

green_start!(main)

fn main() {
    let msgs = rtinstrument::instrument::<green::task::GreenTask>(|| {
        let (tx, rx) = channel();
        for _ in range(0, 10) {
            let tx = tx.clone();
            spawn(proc() {
                println!("baz");
                tx.send(());
            });
        }
        for _ in range(0, 10) {
            rx.recv();
        }
    });

    for msg in msgs.iter() {
        println!("{}", msg);
    }
}
