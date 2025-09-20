# Pipes

Injecting protobuf messages into named pipe communications between SUID processes.

To solve, compile `solver` with `cargo b --release` and upload it with SCP.

Run `solver&` then `/client`. Press buttons to make a move and the solver will eventually inject a jump to the right, bypassing the walls, where we can get the flag.
