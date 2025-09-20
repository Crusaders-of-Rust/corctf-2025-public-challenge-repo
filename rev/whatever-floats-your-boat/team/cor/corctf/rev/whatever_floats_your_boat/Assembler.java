package team.cor.corctf.rev.whatever_floats_your_boat;

import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.io.FileOutputStream;
import java.io.IOException;

import static team.cor.corctf.rev.whatever_floats_your_boat.OpCode.*;

public class Assembler {

    public static void main(String[] args) {
        ByteBuffer buffer = ByteBuffer.allocate(50_000 * Double.BYTES);
        buffer.order(ByteOrder.LITTLE_ENDIAN); // idk why but it is what it is
        while (buffer.remaining() >= Double.BYTES) {
            buffer.putDouble(NOP.encode()); // fill with NOPs
        }
        buffer.rewind();


        int mainIndex = 1000;
        buffer.putDouble(CALL.encode()).putDouble(mainIndex); // call main function
        buffer.putDouble(RET.encode());


        // function: return whether n greater than equal to 1
        int greaterThanOrEquals1Index = getProgramCount(buffer); // where the function starts
        System.out.println("linking greaterThanOrEquals1Index at pc " + greaterThanOrEquals1Index);
        buffer.putDouble(PUSH_CONST.encode()).putDouble(0.000042006942069)
            .putDouble(MAX.encode()) // add minimum threshold to 1) make positive 2) prevent small magnitude errors
            .putDouble(PUSH_CONST.encode()).putDouble(makeDouble(false, -53, 0))
            .putDouble(CLEAR_EXCEPT.encode())
            .putDouble(ADD.encode()) // n + 2^(-53) is inexact if n >= 1 due to exponent
            .putDouble(POP.encode()) // we didn't need the result, only the fpu flags
            .putDouble(PUSH_CONST.encode()).putDouble(1)
            .putDouble(PUSH_CONST.encode()).putDouble(0) // 1, 0
            .putDouble(B_INEXACT.encode()).putDouble(2) // skip 2 instructions if inexact
            .putDouble(SWAP.encode()) // 0, 1
            .putDouble(POP.encode()) // if inexact, keep 1, else keep 0
            .putDouble(RET.encode());
        
        // function: return whether n is exactly equal to 1
        int equals1Index = getProgramCount(buffer); // where the function starts
        System.out.println("linking equals1Index at pc " + equals1Index);
        // we need to normalize our value to check if *exactly* 1
        buffer.putDouble(PUSH_CONST.encode()).putDouble(0.000042006942069)
            .putDouble(MAX.encode()) // add minimum threshold to make positive
            .putDouble(PUSH_CONST.encode()).putDouble(Double.MAX_VALUE)
            .putDouble(SWAP.encode()) // max_value, n
            .putDouble(DUP2.encode()) // max_value, n, max_value, n
            .putDouble(PUSH_CONST.encode()).putDouble(1) // max_value, n, max_value, n, 1
            .putDouble(SWAP.encode())
            .putDouble(DIV.encode()) // max_value, n, max_value, 1/n
            .putDouble(CLEAR_EXCEPT.encode()) // fire incoming !!!!!!!!!!!
            .putDouble(DIV.encode()) // raises fpu exception if 1/n < 1
            .putDouble(POP.encode())
            .putDouble(DIV.encode()) // raises fpu exception if n < 1
            .putDouble(POP.encode())
            .putDouble(B_OVERFLOW.encode()).putDouble(3); // implied OR gate because FPU exception flags are sticky
        
        // no exception case: n == 1
        buffer.putDouble(PUSH_CONST.encode()).putDouble(1)
            .putDouble(RET.encode());
        
        // overflow exception: n != 1
        buffer.putDouble(PUSH_CONST.encode()).putDouble(0)
            .putDouble(RET.encode());
        
        // function: init puzzle
        int initPuzzleIndex = getProgramCount(buffer);
        String puzzle = ".......1.4.........2...........5.4.7..8...3....1.9....3..4..2...5.1........8.6...";
        // for faster solve, use "....84512487512936125963874932651487568247391741398625319475268856129743274836159";
        for (int i = 0; i < puzzle.length(); i++) {
            char c = puzzle.charAt(i);
            if (c == '.') {
                buffer.putDouble(PUSH_CONST.encode()).putDouble(-1.0); // push -1 for empty cell
            } else {
                buffer.putDouble(PUSH_CONST.encode()).putDouble((double) (c - '0')); // push digit
            }
        } // load entire puzzle into stack
        buffer.putDouble(PUSH_CONST.encode()).putDouble(puzzle.length()); // push puzzle length
        
        // loop start: check if TOS is 0; if so, break the loop
        buffer.putDouble(DUP.encode())
            .putDouble(PUSH_CONST.encode()).putDouble(1)
            .putDouble(SWAP.encode()) // n, 1, n
            .putDouble(CLEAR_EXCEPT.encode()) // clear fpu exceptions
            .putDouble(DIV.encode()) // calculate 1 / n, see if it gives div by 0 error
            .putDouble(POP.encode()) // n
            .putDouble(B_DIVBYZERO.encode()).putDouble(6); // skip to end of loop
        
        // loop body
        buffer.putDouble(PUSH_CONST.encode()).putDouble(-1)
            .putDouble(ADD.encode()) // decrement index
            .putDouble(DUP_X1.encode()) // index, val, index
            .putDouble(STORE.encode()) // store the val at index
            .putDouble(B_ALWAYS.encode()).putDouble(-11); // jump to top of loop
        
        // loop end
        buffer.putDouble(POP.encode()); // pop the index
        
        // set up lookup table
        int[] primeNumbers = new int[] { 2, 3, 5, 7, 11, 13, 17, 19, 23 };
        for (int i = 0; i < primeNumbers.length; i++) {
            buffer.putDouble(PUSH_CONST.encode()).putDouble((double) primeNumbers[i])
                .putDouble(PUSH_CONST.encode()).putDouble(100 + i + 1) // push prime number and its index
                .putDouble(STORE.encode()); // store it in the lookup table
        }
        // lookup for -1 cell: garbage value
        buffer.putDouble(PUSH_CONST.encode()).putDouble(-0.0000000000042069)
                .putDouble(PUSH_CONST.encode()).putDouble(99) // push prime number and its index
                .putDouble(STORE.encode()); // store it in the lookup table

        buffer.putDouble(RET.encode()); // return from initPuzzle function

        // function: lookup number
        int lookupNumberIndex = getProgramCount(buffer);
        buffer.putDouble(PUSH_CONST.encode()).putDouble(100)
            .putDouble(ADD.encode())
            .putDouble(LOAD.encode()) // memory[n + 100]
            .putDouble(RET.encode());
        
        // function: get row ratio
        int getRowProductIndex = getProgramCount(buffer);
        System.out.println("linking getRowProductIndex at pc " + getRowProductIndex);
        buffer.putDouble(PUSH_CONST.encode()).putDouble(1)
            .putDouble(PUSH_CONST.encode()).putDouble(200)
            .putDouble(STORE.encode()) // memory[200] = 1, used for product
            .putDouble(PUSH_CONST.encode()).putDouble(9) // n, 9
            .putDouble(DUP_X1.encode()) // 9, n, 9
            .putDouble(MUL.encode()) // 9, 9n
            .putDouble(SWAP.encode()); // 9n, 9
        
        // loop start: check if TOS is 0; if so, break the loop
        buffer.putDouble(DUP.encode())
            .putDouble(PUSH_CONST.encode()).putDouble(1)
            .putDouble(SWAP.encode()) // n, 1, n
            .putDouble(CLEAR_EXCEPT.encode()) // clear fpu exceptions
            .putDouble(DIV.encode()) // calculate 1 / n, see if it gives div by 0 error
            .putDouble(POP.encode()) // n
            .putDouble(B_DIVBYZERO.encode()).putDouble(14); // skip to end of loop
        
        // loop body
        buffer.putDouble(PUSH_CONST.encode()).putDouble(-1)
            .putDouble(ADD.encode()) // decrement index; // 9n, i-1
            .putDouble(DUP2.encode())
            .putDouble(ADD.encode()) // 9n, i-1, 9n + (i-1)
            .putDouble(LOAD.encode()) // 9n, i-1, memory[9n + (i-1)]
            .putDouble(CALL.encode()).putDouble(lookupNumberIndex) // 9n, i-1, prime number
            .putDouble(PUSH_CONST.encode()).putDouble(200)
            .putDouble(DUP_X1.encode()) // 9n, i-1, 200, prime number, 200
            .putDouble(LOAD.encode()) // 9n, i-1, 200, prime number, memory[200]
            .putDouble(MUL.encode()) // 9n, i-1, 200, new_product
            .putDouble(SWAP.encode()) // 9n, i-1, new_product, 200
            .putDouble(STORE.encode()) // store new product in memory[200]
            .putDouble(B_ALWAYS.encode()).putDouble(-19); // jump to top of loop
        
        // loop end
        buffer.putDouble(POP.encode()) // pop the index
            .putDouble(POP.encode()) // pop the 9n
            .putDouble(PUSH_CONST.encode()).putDouble(200)
            .putDouble(LOAD.encode()) // load the product
            .putDouble(PUSH_CONST.encode()).putDouble(223092870) // push the denominator
            .putDouble(DIV.encode()) // divide by 223092870
            .putDouble(CALL.encode()).putDouble(equals1Index) // normalize the ratio
            .putDouble(RET.encode()); // return the boolean
        
        // function: get column ratio
        int getColumnProductIndex = getProgramCount(buffer);
        System.out.println("linking getColumnProductIndex at pc " + getColumnProductIndex);
        buffer.putDouble(PUSH_CONST.encode()).putDouble(1)
            .putDouble(PUSH_CONST.encode()).putDouble(200)
            .putDouble(STORE.encode()) // memory[200] = 1, used for product
            .putDouble(PUSH_CONST.encode()).putDouble(81) // n, 81
            .putDouble(ADD.encode()); // n + 81 ; each time, subtract 9 and check if <= 9

        // loop start: check if TOS is less than 9; if so, break the loop
        buffer.putDouble(DUP.encode())
            .putDouble(PUSH_CONST.encode()).putDouble(9)
            .putDouble(DIV.encode()) // i, i/9
            .putDouble(CALL.encode()).putDouble(greaterThanOrEquals1Index) // check if i/9 >= 1
            .putDouble(PUSH_CONST.encode()).putDouble(1) // i, bool, 1
            .putDouble(SWAP.encode()) // i, 1, bool
            .putDouble(CLEAR_EXCEPT.encode()) // clear fpu exceptions
            .putDouble(DIV.encode()) // calculate 1 / bool, see if it gives div by 0 exception
            .putDouble(POP.encode()) // i
            .putDouble(B_DIVBYZERO.encode()).putDouble(13); // skip to end of loop if i < 9
        
        // loop body
        buffer.putDouble(PUSH_CONST.encode()).putDouble(-9)
            .putDouble(ADD.encode()) // decrement index; // i-9
            .putDouble(DUP.encode())
            .putDouble(LOAD.encode()) // i-9, memory[i-9]
            .putDouble(CALL.encode()).putDouble(lookupNumberIndex) // i-9, prime number
            .putDouble(PUSH_CONST.encode()).putDouble(200)
            .putDouble(DUP_X1.encode()) // i-9, 200, prime, 200
            .putDouble(LOAD.encode()) // i-9, 200, prime, memory[200]
            .putDouble(MUL.encode()) // i-9, 200, new_product
            .putDouble(SWAP.encode()) // i-9, new_product, 200
            .putDouble(STORE.encode()) // store new product in memory[200]
            .putDouble(B_ALWAYS.encode()).putDouble(-21); // jump to top of loop

        // loop end
        buffer.putDouble(POP.encode()) // pop the index
            .putDouble(PUSH_CONST.encode()).putDouble(200)
            .putDouble(LOAD.encode()) // load the product
            .putDouble(PUSH_CONST.encode()).putDouble(223092870) // push the denominator
            .putDouble(DIV.encode()) // divide by 223092870
            .putDouble(CALL.encode()).putDouble(equals1Index) // normalize the ratio
            .putDouble(RET.encode()); // return the boolean

        // function: get square ratio
        int getSquareProductIndex = getProgramCount(buffer);
        System.out.println("linking getSquareProductIndex at pc " + getSquareProductIndex);
        buffer.putDouble(PUSH_CONST.encode()).putDouble(1)
            .putDouble(PUSH_CONST.encode()).putDouble(200)
            .putDouble(STORE.encode()) // memory[200] = 1, used for product
            .putDouble(DUP.encode()) // i is a number [0, 8)
            .putDouble(PUSH_CONST.encode()).putDouble(3)
            .putDouble(DIV.encode())
            .putDouble(FLOOR.encode())
            .putDouble(DUP_X1.encode()) // i//3, i, i//3
            .putDouble(PUSH_CONST.encode()).putDouble(3)
            .putDouble(MUL.encode())
            .putDouble(SUB.encode()) // i//3, i%3 (row_square_idx, col_square_idx)
            .putDouble(PUSH_CONST.encode()).putDouble(3)
            .putDouble(MUL.encode()) // row_square_idx, col_square_idx * 3
            .putDouble(SWAP.encode())
            .putDouble(PUSH_CONST.encode()).putDouble(27) // each 3 rows is 27 offset
            .putDouble(MUL.encode()) // col_square_idx * 3, row_square_idx * 27
            .putDouble(ADD.encode()); // starting tile index
        
        // im lazy gonna inline the loop
        for (int offset : new int[] { 0, 1, 2, 9, 10, 11, 18, 19, 20 }) {
            buffer.putDouble(DUP.encode())
                .putDouble(PUSH_CONST.encode()).putDouble(offset)
                .putDouble(ADD.encode())
                .putDouble(LOAD.encode())
                .putDouble(CALL.encode()).putDouble(lookupNumberIndex)
                .putDouble(PUSH_CONST.encode()).putDouble(200)
                .putDouble(DUP_X1.encode()) // 200, prime number, 200
                .putDouble(LOAD.encode()) // 200, prime number, product
                .putDouble(MUL.encode()) // 200, new product
                .putDouble(SWAP.encode()) // new product, 200
                .putDouble(STORE.encode()); // memory[200] = new product            
        }

        buffer.putDouble(POP.encode())
            .putDouble(PUSH_CONST.encode()).putDouble(200)
            .putDouble(LOAD.encode())
            .putDouble(PUSH_CONST.encode()).putDouble(223092870) // push the denominator
            .putDouble(DIV.encode())
            .putDouble(CALL.encode()).putDouble(equals1Index) // normalize the ratio
            .putDouble(RET.encode()); // return the boolean

        // function: check solution
        // for each set of 9 cells, look up into primes and compute product
        // then divide that product by 223092870, the intended product
        // then, we get a ratio that represents whether it solved correctly
        // multiply all those ratios for each set and see if it >= 1 using that function.
        int checkSolutionIndex = getProgramCount(buffer);
        System.out.println("linking checkSolutionIndex at pc " + checkSolutionIndex);
        buffer.putDouble(PUSH_CONST.encode()).putDouble(9); // loop from range (0,9)

        // loop start: check if TOS is less than 1; if so, break the loop
        buffer.putDouble(DUP.encode())
            .putDouble(CALL.encode()).putDouble(greaterThanOrEquals1Index) // check if i >= 1
            .putDouble(PUSH_CONST.encode()).putDouble(1) // i, bool, 1
            .putDouble(SWAP.encode()) // i, 1, bool
            .putDouble(CLEAR_EXCEPT.encode()) // clear fpu exceptions
            .putDouble(DIV.encode()) // calculate 1 / bool, see if it gives div by 0 exception
            .putDouble(POP.encode()) // i
            .putDouble(B_DIVBYZERO.encode()).putDouble(13); // skip to end of loop if i < 1
        
        // loop body: decrement i and compute check-products
        buffer.putDouble(PUSH_CONST.encode()).putDouble(1)
            .putDouble(SUB.encode()) // i--
            .putDouble(DUP.encode())
            .putDouble(CALL.encode()).putDouble(getRowProductIndex)
            .putDouble(SWAP.encode())
            .putDouble(DUP.encode())
            .putDouble(CALL.encode()).putDouble(getColumnProductIndex)
            .putDouble(SWAP.encode())
            .putDouble(DUP.encode())
            .putDouble(CALL.encode()).putDouble(getSquareProductIndex)
            .putDouble(SWAP.encode())
            .putDouble(B_ALWAYS.encode()).putDouble(-19); // jump to top of loop
        
        // loop end
        buffer.putDouble(POP.encode());

        // multiply all the check-products !!
        buffer.putDouble(PUSH_CONST.encode()).putDouble(1.0);
        for (int i=0; i<27; i++) {
            buffer.putDouble(MUL.encode());
        }

        // we need to normalize our value to check if *exactly* 1
        buffer.putDouble(CALL.encode()).putDouble(equals1Index)
            .putDouble(RET.encode());

        // function: attempt_solve_recursive(index)
        // recursively brute forces the solver with index++ until index == 81.
        // where it does a final check and returns boolean
        int attemptSolveRecursiveIndex = getProgramCount(buffer);
        System.out.println("linking attemptSolveRecursiveIndex at pc " + attemptSolveRecursiveIndex);
        // if index == 81, simply do a check and return
        buffer.putDouble(DUP.encode()) // index, index
            .putDouble(PUSH_CONST.encode()).putDouble(81) // index, index, 81
            .putDouble(DIV.encode()) // index, index/81
            .putDouble(CALL.encode()).putDouble(equals1Index) // check if index == 81
            .putDouble(PUSH_CONST.encode()).putDouble(1)
            .putDouble(SWAP.encode()) // index, 1, bool
            .putDouble(CLEAR_EXCEPT.encode())
            .putDouble(DIV.encode())
            .putDouble(POP.encode()) // we dont need result, just the exception flag
            .putDouble(B_DIVBYZERO.encode()).putDouble(4); // skip branch if not index == 81
        
        // base case: just check if solved and return result (also pop the index)
        buffer.putDouble(POP.encode())
            .putDouble(CALL.encode()).putDouble(checkSolutionIndex)
            .putDouble(RET.encode());
        
        // otherwise, do recursive guessing
        // first, check if the cell is empty (< 1)
        buffer.putDouble(DUP.encode()) // index, index
            .putDouble(LOAD.encode()) // index, curr_val
            .putDouble(CALL.encode()).putDouble(greaterThanOrEquals1Index) // check if i >= 1
            .putDouble(PUSH_CONST.encode()).putDouble(1) // index, bool, 1
            .putDouble(SWAP.encode()) // index, 1, bool
            .putDouble(CLEAR_EXCEPT.encode())
            .putDouble(DIV.encode())
            .putDouble(POP.encode()) // we dont need result, just the exception flag
            .putDouble(B_DIVBYZERO.encode()).putDouble(5); // skip branch if i still needs guessing; // index
        
        // case: there is already a hint at this cell
        // just recurse next without guesses and return that result
        buffer.putDouble(PUSH_CONST.encode()).putDouble(1)
            .putDouble(ADD.encode())
            .putDouble(CALL.encode()).putDouble(attemptSolveRecursiveIndex) // recurse (index + 1)
            .putDouble(RET.encode());
        
        // regular case: make guesses [1, 9], break loop if returned true
        buffer.putDouble(PUSH_CONST.encode()).putDouble(9); // index, guess

        // loop start: check if guess is still >= 1
        buffer.putDouble(DUP.encode()) // index, guess, guess
            .putDouble(CALL.encode()).putDouble(greaterThanOrEquals1Index) // check if guess >= 1
            .putDouble(PUSH_CONST.encode()).putDouble(1) // index, guess, bool, 1
            .putDouble(SWAP.encode()) // index, guess, 1, bool
            .putDouble(CLEAR_EXCEPT.encode())
            .putDouble(DIV.encode())
            .putDouble(POP.encode()) // we dont need result, just the exception flag
            .putDouble(B_DIVBYZERO.encode()).putDouble(22); // break loop if guess is zero now; // index, guess
        
        // loop body: attempt guess and then decrement index
        buffer.putDouble(DUP2.encode()) // index, guess, index, guess
            .putDouble(SWAP.encode()) // index, guess, guess, index                        
            .putDouble(STORE.encode()) // index, guess
            .putDouble(SWAP.encode()) // guess, index
            .putDouble(DUP_X1.encode()) // index, guess, index
            .putDouble(PUSH_CONST.encode()).putDouble(1)
            .putDouble(ADD.encode()) // index, guess, index + 1
            .putDouble(CALL.encode()).putDouble(attemptSolveRecursiveIndex) // recurse (index + 1); puts a bool on tos
            .putDouble(PUSH_CONST.encode()).putDouble(1) // index, guess, bool, 1
            .putDouble(SWAP.encode()) // index, guess, 1, bool
            .putDouble(CLEAR_EXCEPT.encode())
            .putDouble(DIV.encode())
            .putDouble(POP.encode()) // we dont need result, just the exception flag
            .putDouble(B_DIVBYZERO.encode()).putDouble(5); // skip this if guess failed; // index, guess
        
        // success! we're done now
        // pop index, guess; return 1
        buffer.putDouble(POP.encode())
            .putDouble(POP.encode())
            .putDouble(PUSH_CONST.encode()).putDouble(1) // true
            .putDouble(RET.encode());
        
        // otherwise, the guess didn't work. we need to decrement guess and loop again
        buffer.putDouble(PUSH_CONST.encode()).putDouble(-1) // index, guess, -1
            .putDouble(ADD.encode()) // guess--
            .putDouble(B_ALWAYS.encode()).putDouble(-28);

        // loop end
        // all guesses failed; return false
        // restore the value as -1 at index
        // pop index, guess; return 0
        buffer.putDouble(POP.encode()) // index
            .putDouble(PUSH_CONST.encode()).putDouble(-1) // index, -1
            .putDouble(SWAP.encode()) // -1, index
            .putDouble(STORE.encode()) // memory[index] = -1
            .putDouble(PUSH_CONST.encode()).putDouble(0) // false
            .putDouble(RET.encode());

        // function: attempt_solve() returns whether it worked
        int attemptSolveIndex = getProgramCount(buffer);
        System.out.println("linking attemptSolveIndex at pc " + attemptSolveIndex);
        buffer.putDouble(PUSH_CONST.encode()).putDouble(0)
            .putDouble(CALL.encode()).putDouble(attemptSolveRecursiveIndex) // call algorithm starting with 0
            .putDouble(RET.encode());
        
        // function: hash_board() loops through the board and returns a hash
        // loops through from index 0 to 81, multiplying and adding
        int hashBoardIndex = getProgramCount(buffer);
        System.out.println("linking hashBoardIndex at pc " + hashBoardIndex);
        buffer.putDouble(PUSH_CONST.encode()).putDouble(0.00000000069) // seed value
            .putDouble(PUSH_CONST.encode()).putDouble(200) // memory address
            .putDouble(STORE.encode()) // memory[200] = 0.00000000069, used for hash
            .putDouble(PUSH_CONST.encode()).putDouble(0); // loop from 0 to 81

        // loop start: check if TOS is greater than or equals 81; if so, break the loop
        buffer.putDouble(DUP.encode())
            .putDouble(PUSH_CONST.encode()).putDouble(81)
            .putDouble(DIV.encode())
            .putDouble(CALL.encode()).putDouble(greaterThanOrEquals1Index) // check if i/81 >= 1
            .putDouble(PUSH_CONST.encode()).putDouble(1) // i, bool, 1
            .putDouble(SWAP.encode()) // i, 1, bool
            .putDouble(SUB.encode()) // i, !bool
            .putDouble(PUSH_CONST.encode()).putDouble(1) // i, !bool, 1
            .putDouble(SWAP.encode()) // i, 1, !bool
            .putDouble(CLEAR_EXCEPT.encode())
            .putDouble(DIV.encode())
            .putDouble(POP.encode()) // we dont need result, just the exception flag
            .putDouble(B_DIVBYZERO.encode()).putDouble(17); // break loop if i/81 >= 1 is true

        // loop body: hash the current board state
        buffer.putDouble(DUP.encode())
            .putDouble(LOAD.encode()) // i, memory[i]
            .putDouble(PUSH_CONST.encode()).putDouble(200)
            .putDouble(DUP_X1.encode())
            .putDouble(LOAD.encode()) // i, 200, memory[i], memory[200]
            .putDouble(PUSH_CONST.encode()).putDouble(Math.PI)
            .putDouble(MUL.encode()) // i, 200, memory[i], memory[200] * Math.PI
            .putDouble(SWAP.encode()) // i, 200, memory[200] * Math.PI, memory[i]
            .putDouble(MUL.encode()) // i, 200, memory[200] * Math.PI * memory[i]
            .putDouble(PUSH_CONST.encode()).putDouble(Math.E)
            .putDouble(ADD.encode()) // i, 200, memory[200] * Math.PI * memory[i] + e
            .putDouble(SWAP.encode()) // i, new_hash, 200
            .putDouble(STORE.encode()) // store new_hash in memory[200]
            .putDouble(PUSH_CONST.encode()).putDouble(1) // i, 1
            .putDouble(ADD.encode()) // i + 1
            .putDouble(B_ALWAYS.encode()).putDouble(-28); // jump back to loop start

        // loop end
        buffer.putDouble(PUSH_CONST.encode()).putDouble(200)
            .putDouble(LOAD.encode())
            .putDouble(RET.encode());

        // function: main
        int usedProgramCount = getProgramCount(buffer);
        System.out.println("Program functions used up to program count " + usedProgramCount);
        System.out.println("Linking main at " + mainIndex);
        if (usedProgramCount > mainIndex) {
            throw new IllegalStateException("Invalid position to link main!");
        }
        buffer.position(indexOf(buffer, mainIndex));

        String boatImage = """
\u001B[0m
                                                ....
                                            ..';::c:;,'.
                                        ..,;:cccccccccc:;,'.
                                    .',;::ccc\u001B[31mcorctf{...}\u001B[0mc:c::;,'.
                                  .....';:ccccccccccc:ccc::;,'...
                           .;kOo. .',.....',;:::::cccc:;,'.......
                         'oOdclxkd..,,,,'.....',;::;,............
                      'lkdl:;;:oxd...',,'..,,'................... ,.
                   .ckdl::;;cll:;;.....;kdc'',,;'............. .. xkxo;.
        ';.     .:kOxoll:;;;;;,,;,..''':oc,.oxkd' ........ ...... cxO0xl'
     .:c;,::. ;xOkOOkdl:;;;;;;;,,,..,,,;c;'.dOOO' ............':lxOOxoc;.
  .:oc,,,,,:dO0Oko:;;;;;;;;,,,,,,,.',,;;;,..:000c ........';cxOOxoc:;;;;.
,ol:;;;;lxO0kdl:;;,,;,,,,,,,,,,,:l..,,,,,,,,,xkOx.....':ok0Oxlc:;;;;;;;,.
.,::;;,cdkOo:;;;;,,,,,,;;;,,:ldkkx,...',,,,,,oOOk'.;ok0Oxoc;;,;;;;;,,,,,.
   .:;;:dxkkkkxl;,,,,,,;:ldkkOkkkkkko:'...',,;0OxxOOxoc:;;;;;,,,,,,;;,,
     .;:oolldxkO0OxlcldkkOOkxxxkkxdddxkkoc,;okO0kkl;;;;;;;;,,,,;;;,,.
       .lllclooodxkO000OkxxxkkxdddxkkkkO0Okkxoo0OOo;;;,,,,,;;;,,,,;'.
       .llcclloooooodxkO00OxdddxkOOO00OOxoc;;;;OOOk,,,,;;;,,,,,,..
       .llollcccloooloooddxkO0kxOOOOxoc;;;,,;;,d0Ok:,,,,,,''..
         .;llllollccllooocoodc;codc;;;;;;;,,,,,:00Od,,,,..
          .cllcclllllccclcllo;;:lo;;;;,,,,,;;,,,x0xd,.
             .,::clllllolcccc,,;cl;,,,;;;,,,,;;,:kxx:
                 .,cllllllolc;;:ll;;;;,,,','..   OOOO
                     .,cc:ccc;;;:c;,,,,'.        c0O0,
                         .'clll:,,;,..           .0O0x
                             .,;..                ,,x0.
                                                    .;.
                                                     ..
\u001B[0m
""";
        String titleCard = """
                ____   ___    _  _____        __     ____  __ 
               | __ ) / _ \\  / \\|_   _|       \\ \\   / /  \\/  |
               |  _ \\| | | |/ _ \\ | |          \\ \\ / /| |\\/| |
               | |_) | |_| / ___ \\| |           \\ V / | |  | |
               |____/ \\___/_/   \\_\\_|   _____    \\_/  |_|  |_|
                                       |_____|                
\n""";

        System.out.print(boatImage);
        printString(buffer, boatImage);
        System.out.print(titleCard);
        printString(buffer, titleCard);
        printString(buffer, "\n");
        buffer.putDouble(CALL.encode()).putDouble(initPuzzleIndex); // call initPuzzle function
        printString(buffer, "Running corCTF flag cracker.\n\n");
        
        printString(buffer, "Hang tight, I'm using hardware acceleration to fulfill out your request...\n\n");
        
        // loop delay
        buffer.putDouble(PUSH_CONST.encode()).putDouble(20_000_000);
        // loop start: check if TOS is less than 1; if so, break the loop
        buffer.putDouble(DUP.encode())
            .putDouble(CALL.encode()).putDouble(greaterThanOrEquals1Index) // check if i >= 1
            .putDouble(PUSH_CONST.encode()).putDouble(1) // i, bool, 1
            .putDouble(SWAP.encode()) // i, 1, bool
            .putDouble(CLEAR_EXCEPT.encode()) // clear fpu exceptions
            .putDouble(DIV.encode()) // calculate 1 / bool, see if it gives div by 0 exception
            .putDouble(POP.encode()) // i
            .putDouble(B_DIVBYZERO.encode()).putDouble(5); // skip to end of loop if i < 1
        // loop body
        buffer.putDouble(PUSH_CONST.encode()).putDouble(-1)
            .putDouble(ADD.encode()) // decrement index
            .putDouble(NOP.encode())
            .putDouble(B_ALWAYS.encode()).putDouble(-11); // jump to top of loop
        
        printString(buffer, "Thanks for waiting! Just a bit longer...\n\n");

        // loop delay
        buffer.putDouble(PUSH_CONST.encode()).putDouble(10_000_000);
        // loop start: check if TOS is less than 1; if so, break the loop
        buffer.putDouble(DUP.encode())
            .putDouble(CALL.encode()).putDouble(greaterThanOrEquals1Index) // check if i >= 1
            .putDouble(PUSH_CONST.encode()).putDouble(1) // i, bool, 1
            .putDouble(SWAP.encode()) // i, 1, bool
            .putDouble(CLEAR_EXCEPT.encode()) // clear fpu exceptions
            .putDouble(DIV.encode()) // calculate 1 / bool, see if it gives div by 0 exception
            .putDouble(POP.encode()) // i
            .putDouble(B_DIVBYZERO.encode()).putDouble(5); // skip to end of loop if i < 1
        // loop body
        buffer.putDouble(PUSH_CONST.encode()).putDouble(-1)
            .putDouble(ADD.encode()) // decrement index
            .putDouble(NOP.encode())
            .putDouble(B_ALWAYS.encode()).putDouble(-11); // jump to top of loop

        printString(buffer, "Estimated time remaining: ");
        buffer.putDouble(PUSH_CONST.encode()).putDouble(2.218531200133700e+57).putDouble(PRINT_FLOAT.encode());
        printString(buffer, " seconds\n\n");

        buffer.putDouble(CALL.encode()).putDouble(attemptSolveIndex); // call attemptSolve function

        printString(buffer, "Cracker finished!\n");
        printString(buffer, "Found viable solution: ");
        buffer.putDouble(PRINT_FLOAT.encode()); // prints 0 or 1 for solve status
        printString(buffer, "\n");

        buffer.putDouble(CALL.encode()).putDouble(hashBoardIndex); // call hashBoard function
        printString(buffer, "\nFlag: \n");
        printString(buffer, "corctf{g3ntly_d0wn_th3_StR3AM_");
        buffer.putDouble(PRINT_FLOAT.encode()); // prints the board hash
        printString(buffer, "FPU_h4cks}\n");
        printString(buffer, "\nThank you for your patience!\n");



        buffer.putDouble(RET.encode()); // return from main function

        

        buffer.flip();

        String filePath = "float_program.bin";
        try (FileOutputStream out = new FileOutputStream(filePath)) {
            out.getChannel().write(buffer);
            System.out.println("Bytecode written to " + filePath);
        } catch (IOException e) {
            e.printStackTrace();
        }
    }

    public static void printString(ByteBuffer buf, String str) {
        for (char c : str.toCharArray()) {
            buf.putDouble(PUSH_CONST.encode()).putDouble((double) c).putDouble(PRINT_CHAR.encode());
        }
    }

    public static double makeDouble(boolean sign, int exponent, long mantissa) {
        if (mantissa < 0 || mantissa > ((1L << 52) - 1)) {
            throw new IllegalArgumentException("Mantissa must be 52 bits");
        }

        // Bias exponent by 1023 for IEEE 754
        long biasedExp = (long) (exponent + 1023);
        if (biasedExp < 0 || biasedExp > 0x7FF) {
            throw new IllegalArgumentException("Exponent out of range after biasing");
        }

        long signBit = sign ? (1L << 63) : 0L;
        long expBits = biasedExp << 52;
        long bits = signBit | expBits | mantissa;

        return Double.longBitsToDouble(bits);
    }

    // steps through until it finds the appropriate location to link an insn
    public static int indexOf(ByteBuffer buf, int programCount) {
        int index = 0;
        for (int i = 0; i < programCount; i++) {
            double val = buf.getDouble(index++ * Double.BYTES);
            OpCode opCode = OpCode.values()[(int) val];
            if (opCode.hasData()) {
                index++; // skip another index for the data
            }
        }
        return index * Double.BYTES;
    }

    // finds the program count for the current buffer position
    public static int getProgramCount(ByteBuffer buf) {
        int pos = buf.position();
        int count = 0;
        for (int i = 0; i < pos / Double.BYTES; i++) {
            double val = buf.getDouble(i * Double.BYTES);
            OpCode opCode = OpCode.values()[(int) val];
            // System.out.println(opCode); // to debug
            if (opCode.hasData()) {
                i++; // skip another index for the data
            }
            count++;
        }
        return count;
    }
}