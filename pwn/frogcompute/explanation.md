# Frogcompute

Frogcompute is an easy pwn challenge.
Players were given the program binary which was being used on the server.
The program is a modified brainfuck interpreter, with a special "mmio" procedure 
that will print the flag if cell `0x69420` is written to. 

Since submitted bf programs are limited to 90 characters, and the system comes with 
out-of-bounds checking, it is obviously impossible to trigger this normally.
In the original C code, the OOB check is written fairly suspiciously: it only 
fails if `(ref_addr > end_addr)`. This means that underflow is possible 
by shifting to the left.

The exploit requires users to shift to the left, first underflowing the buffer, 
and then increment the buffer size variable. Then, they can shift to the right 
and overwrite the return address on the stack, performing a ret2win technique.

Due to the nature of the bf language, players can directly bypass the stack canary
and ASLR by simply just incrementing the correct part of the return address by the 
function's offset. 
Due to the small character limit, players had to optimize their payloads using 
some simple bf algorithms. My intended solution is available in `solve.sh`.
