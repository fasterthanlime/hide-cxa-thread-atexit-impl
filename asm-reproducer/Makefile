all:
	gcc -o stub.o -c stub.S
	gcc -o ruststd.o -c ruststd.S
	ld -shared -o libfoobar.so --version-script=list stub.o ruststd.o

clean:
	rm -f *.o *.so
