# corchat-v4

goal: cause the app to panic, which is caught and prints the flag

Rust will panic if it writes to a closed pipe. Solve with

```
( corchat 2>&1 1>&3 | head -n 1 ) 3>&1
```
To redirect stderr to head (which is closed after one line), and stdout to fd 3 then back to 1 to read output.

Inside the challenge, run `enter foo` twice to print two lines of errors, panicking and getting the flag.
