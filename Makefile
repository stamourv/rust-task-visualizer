all:
	rustc rtinstrument.rs
	rustc foo.rs -L .
	./foo | racket rust-visualizer.rkt &
