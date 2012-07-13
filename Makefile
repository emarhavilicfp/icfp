CFLAGS=--static
LDFLAGS=-lpthread -lrt -ldl

# Remember to add modules for your .rs files in icfp.rc too!
icfp: icfp.rc hello.rs
	rustc -c icfp.rc
	g++ -o ./bin/icfp ${CFLAGS} icfp.o lib/*.o lib/*.a ${LDFLAGS}

clean:
	rm -f icfp.o ./bin/icfp
