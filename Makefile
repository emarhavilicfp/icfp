CFLAGS=--static
LDFLAGS=-lpthread -lrt -ldl
MODE ?= dynamic

all: bin/icfp bin/testseq

C_SRC = c_signal.c
C_OBJ = $(C_SRC:.c=.o)

ICFP_SRC = icfp.rc \
	   driver.rs \
	   state.rs \
	   path.rs \
	   pattern.rs \
	   evaluate.rs \
	   dlist.rs \
	   heuristics.rs \
	   play.rs \
	   fuzzer.rs \
	   testseq.rc \
	   testseq_driver.rs

# Remember to add modules for your .rs files in icfp.rc too!
ifeq ($(MODE),dynamic)

bin/icfp: $(ICFP_SRC) $(C_OBJ)
	mkdir -p ./bin
	rustc icfp.rc -o ./bin/icfp

bin/testseq: $(ICFP_SRC) $(C_OBJ)
	mkdir -p ./bin
	rustc testseq.rc -o ./bin/testseq

else

bin/icfp: $(ICFP_SRC) $(C_OBJ)
	rustc -c icfp.rc
	mkdir -p ./bin
	g++ -o ./bin/icfp ${CFLAGS} icfp.o c_signal.o lib/*.o lib/*.a ${LDFLAGS}

bin/testseq:

endif

check: $(ICFP_SRC) $(C_OBJ)
	mkdir -p ./bin
	rustc icfp.rc -o ./bin/icfp-test --test
	./bin/icfp-test

clean:
	rm -f icfp.o ./bin/icfp $(C_OBJ)

BUILD=$(shell date +icfp-%Y%m%d-%H%M)

PKGFILES=pkg/PACKAGES_TESTING pkg/README pkg/install pkg/lifter

ifneq ($(shell uname -n),icfp)
pkg:
	@echo "You *must* build this package on the VM."; exit 1
else
pkg: bin/icfp
	rm -rf $(BUILD)
	mkdir $(BUILD)
	cp $(PKGFILES) $(BUILD)
	cp -R patterns $(BUILD)
	cp bin/icfp $(BUILD)
	mkdir $(BUILD)/lib
	cp `ldd bin/icfp | grep /usr/local/lib/rustc | cut -d' ' -f3` $(BUILD)/lib
	git archive --format tar --prefix src/ HEAD | tar x -C $(BUILD)
	tar czvf $(BUILD).tar.gz -C $(BUILD) .
#	rm -rf $(BUILD)
endif

.PHONY: pkg

.phony: etags
etags:
	ctags -e -f TAGS --options=./etc/ctags.rust -R .
