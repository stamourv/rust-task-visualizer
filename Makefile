RUSTC := rustc
BUILD := build
LIB := $(BUILD)/$(shell $(RUSTC) --print-file-name rtinstrument.rs)
TESTS := foo msgsend-pipes-shared msgsend-ring-mutex-arcs \
	shootout-threadring
TESTS := $(TESTS:%=$(BUILD)/%)

all: $(TESTS)

$(LIB): rtinstrument.rs
	@mkdir -p $(@D)
	$(RUSTC) $< --out-dir $(@D) -O

$(TESTS): $(BUILD)/%: %.rs $(LIB)
	$(RUSTC) $< --out-dir $(@D) -L $(BUILD)

clean:
	rm -rf build
