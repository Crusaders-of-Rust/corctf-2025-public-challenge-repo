# Purely Functional Oop

This challenge introduced a novel system which transpiled an object-oriented programming language 
into a google sheets formula. It used a smalltalk-like message-passing system and google sheet's 
builtin LAMBDA functions.

Programs were linked against a `Challenge` class which could be instantiated to attempt to earn the flag. 
The challenge required users to submit objects which would satisfy Fermat's Last Theorem for n=3, which 
mathematically should be impossible.

## The Solution

The intended solution was to find a bug in the compiler: `_rawVal`, which was used for 
wrapping numbers, was not listed as a reserved keyword. This means that users could define 
custom datatypes which were not truthful in their behaviors. My exploit involved a self-propagating 
boolean, who would always return itself even if fed into an and gate with another boolean.

```js
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
```

However, many solvers found a cheese solution, exploiting google sheets' type confusion, getting 
the flag using `new Challenge("0", "0", "0")`. I should have used the builtin `ISNUMBER` function 
to assert the integrity of the raw datatypes, but oh well. In any case, solvers had to exploit 
weaknesses in the type systems of the platform.
