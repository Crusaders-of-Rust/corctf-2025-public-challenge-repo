# Handles the challenge functions
FLAG = open("flag.txt").read().strip()

# The code that gets linked with the user submission
CHALL_CODE = """
    fn _pow(curr_val, exp_base, exponent) {
        let done = exponent.equals(0);
        return done ? curr_val : _pow(curr_val.times(exp_base), exp_base, exponent.minus(1));
    }

    fn pow(exp_base, exponent) {
        return _pow(1, exp_base, exponent);
    }

    fn nonzero(x) {
        return x.notEquals(0).and(0.notEquals(x));
    }

    fn isInteger(x) {
        let sign_val = x.greaterThan(0) ? 1 : -1;
        let absVal = sign_val.times(x);
        
        let exactlyZero = absVal.equals(0);
        let isDecimal = absVal.greaterThan(0).and(absVal.lessThan(1));
        return exactlyZero ? true : (isDecimal ? false : isInteger(absVal.minus(1)));
    }

    class Challenge {
        constructor() {
            let this.n = 3;
            let this.flag = """ + f'"{FLAG}"' + """;
        }

        fn verifySolution(a, b, c) {
            let nonzeroInput = nonzero(a).and(nonzero(b)).and(nonzero(c));
            let integerInput = isInteger(a).and(isInteger(b)).and(isInteger(c));
            let satisfiesTheorem = pow(a, this.n).plus(pow(b, this.n)).equals(pow(c, this.n));
            
            let accepted = nonzeroInput.and(integerInput).and(satisfiesTheorem);
            return accepted ? this.flag : "Solution rejected";
        }
    }

"""

from compiler import compile_code
from formula_executor import execute_formula

def run_submission(user_code):
    challenge_code = CHALL_CODE + str(user_code)
    compiled_code = compile_code(challenge_code)
    result = execute_formula(compiled_code)
    # print("Result of executing the compiled code:")
    # print(result)
    return result

