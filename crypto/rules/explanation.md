# Rules

Honestly, this is barely even a crypto challenge. I kinda whipped this together 
because we didn't have enough challenges. 

```py
def rule(data):
    left = rotate_left(data)
    right = rotate_right(data)

    return ~((left & right & data) | (~left & ~right & ~data) | (~left & right & ~data))
```

As the challenge name implies, the `rule` function is actually an 
emulation of the [Rule 110 Cellular Automaton](https://en.wikipedia.org/wiki/Rule_110#Spaceships_in_Rule_110) 
on a circular bitarray. The goal was to find an input sequence that displayed 
periodicity in its output, so that guessing the output over a random amount of iterations 
becomes feasible.

My intended solution uses the "ether" pattern which repeats itself exactly 
after a period of 7 iterations, yielding a 1/7 success rate in guessing. 
It seems that other solvers used large arrays of the same value, which tended to 
also repeat itself pretty easily. I probably should have rejected inputs 
that consisted entirely of the same number, but oh well.
