RUSTC := rustc
BUILD := build
LIB := $(BUILD)/$(shell $(RUSTC) --crate-file-name rtinstrument.rs)
TESTS := foo msgsend-pipes-shared msgsend-ring-mutex-arcs
TESTS := $(TESTS:%=$(BUILD)/%)

all: $(TESTS)

$(LIB): rtinstrument.rs
	@mkdir -p $(@D)
	$(RUSTC) $< --out-dir $(@D) -O

$(TESTS): $(BUILD)/%: %.rs $(LIB)
	$(RUSTC) $< --out-dir $(@D) -L $(BUILD)

# all:
# 	rustc rtinstrument.rs
# 	rustc foo.rs -L .
# 	./foo | racket rust-visualizer.rkt &
