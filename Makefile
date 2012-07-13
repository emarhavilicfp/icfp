CFLAGS=--static
LDFLAGS=-lpthread -lrt -ldl

icfp: static

# Remember to add modules for your .rs files in icfp.rc too!
static: icfp.rc hello.rs
	rustc -c icfp.rc
	mkdir -p ./bin
	g++ -o ./bin/icfp ${CFLAGS} icfp.o lib/*.o lib/*.a ${LDFLAGS}

dynamic: icfp.rc hello.rs
	mkdir -p ./bin
	rustc icfp.rc -o ./bin/icfp

clean:
	rm -f icfp.o ./bin/icfp
