#!/usr/bin/env python3

from submission_verifier import run_submission

print("The Purely Functional OOP Lang")
print("Is a purely object-oriented language that compiles to purely functional Google Sheets formulae.")
print("Declare variables, functions, and classes using `let`, `fn`, and `class`.")
print("The last line of your code should be a return statement, representing the program output.")
print("Your code is also linked with a challenge, which you can attempt to solve to obtain the flag.")
print("\nEnter lines of code, with the last line containing only the string 'EOF': ")

user_code_lines = []
while True:
    line = input()
    if line.strip() == "EOF":
        break
    user_code_lines.append(line)

user_code = "\n".join(user_code_lines)

print("\nRunning your code...")

try:
    result = run_submission(user_code)
    print("Compiled and ran! Execution result: ")
    print(result)
except Exception:
    print("Error occurred while compiling or evaluating user code")
    print("Exiting")