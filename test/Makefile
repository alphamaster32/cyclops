default: test 

test.o: test.c
	riscv32-unknown-elf-gcc -c test.c -o test.o -static

test: test.o
	riscv32-unknown-elf-gcc test.o -o test

clean:
	-rm -f test.o
	-rm -f test 
