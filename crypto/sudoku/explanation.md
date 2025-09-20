# Sudoku

This challenge involves a zero-knowledge proof using a 9-coloring graph 
problem for an unsolvable sudoku puzzle. In order to get the flag, 
players had to convince the verifier that it had correctly solved 
the puzzle.

The problem with the protocol is that its commitment system 
accepts variable-length nonces due to its usage of string hashing, 
as well as a conveniently-placed `-` symbol before the nonce in the 
format string `f"{color_name}-{nonce} and some salt for fun"`.

Under this system, a color name of `a` with a nonce of `-1` 
could be confused with a color name of `a-` with a nonce of `1`, 
allowing a commitment to have multiple underlying values when revealed.

The server performs 8192 rounds of verification, which can easily exceed 
the session timeout without any networking optimisation. In our solution, 
we were able to speed up our solve script by orders of magnitude by 
only occasionally performing a `recvuntil()` on the socket. 

```py
for i in range(NUM_ROUNDS):
    if i % 100 == 0:
        s.recvuntil(f"Round {i}")
        print("Received prompt for round", i)
    s.sendline(commitment_to_json(board))
    s.sendline(commitment_to_json(["a", "a-", *[f"wesh{i}" for i in range(7)]]))
    s.sendline(reveal_to_json([entry_1, entry_2]))
    if i % 1000 == 0:
        print(f"Round {i} passed")

s.recvuntil("All rounds passed")
print(s.recvall().decode())
```

