CFLAGS=--static
LDFLAGS=-lpthread -lrt -ldl
MODE ?= dynamic

all: $(MODE)

ICFP_SRC = icfp.rc hello.rs state.rs path.rs pattern.rs evaluate.rs dlist.rs

# Remember to add modules for your .rs files in icfp.rc too!
static: $(ICFP_SRC)
	rustc -c icfp.rc
	mkdir -p ./bin
	g++ -o ./bin/icfp ${CFLAGS} icfp.o lib/*.o lib/*.a ${LDFLAGS}

dynamic: $(ICFP_SRC)
	mkdir -p ./bin
	rustc icfp.rc -o ./bin/icfp

check: $(ICFP_SRC)
	mkdir -p ./bin
	rustc icfp.rc -o ./bin/icfp-test --test
	./bin/icfp-test

clean:
	rm -f icfp.o ./bin/icfp
