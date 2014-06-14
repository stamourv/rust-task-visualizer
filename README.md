Rust Task Visualizer
====================

Prototype task visualizer for Rust tasks, based on Racket's futures
visualizer.

Shows which tasks run on which threads, and when.

To run:
* `./my-rust-program | racket rust-visualizer.rkt &`

The UI is the same as Racket's futures visualizer and thus uses
Racket's terminology. Not all the information makes sense for Rust, and
some terms are different.

In general:
* Rust tasks map to Racket futures
* Rust threads map to Racket processes
* `sync` events correspond to tasks being descheduled
