# this is a spurious comment
CFLAGS=--static
LDFLAGS=-lpthread -lrt -ldl

# Remember to add modules for your .rs files in icfp.rc too!
icfp: icfp.rc hello.rs
	rustc -c icfp.rc
	mkdir -p ./bin
	g++ -o ./bin/icfp ${CFLAGS} icfp.o lib/*.o lib/*.a ${LDFLAGS}

clean:
	rm -f icfp.o ./bin/icfp
