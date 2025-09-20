from pwn import *
import os

# context.log_level = "debug"

os.system("anchor build")

with open("target/deploy/player.so", "rb") as f:
    program = f.read()

r = remote(args.HOST or "localhost", args.PORT or 5001)

print(r.recvuntil(b"program pubkey: ").decode())
r.sendline(b"BujTCzJfF399XRtT2vwqztB8ihhhEKQkwYnyFNs2Kq7S")

print(r.recvuntil(b"program len: ").decode())
r.sendline(str(len(program)).encode())

r.send(program)

r.interactive()