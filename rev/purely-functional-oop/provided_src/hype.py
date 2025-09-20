# used for sneak peek tweet, not actually part of the chall lol
# but take this as a little language tutorial I guess :)

hype_code = """
class CorCTF {
    constructor(year, flagBody) {
        let this.year = year;
        let this.flagBody = flagBody;
    }

    fn flag() {
        return "corctf{"
      		.concat(this.flagBody)
        	.concat("}");
    }

    fn message() {
        return "Are you ready for corCTF "
      	    .concat(this.year)
      		.concat("??");
    }
}

let ctf = new CorCTF(2025, "aw3some_ch411eng3s_c0m1ng");

return ctf.message().concat(ctf.flag());
"""

from compiler import compile_code
from formula_executor import execute_formula

compiled_code = compile_code(hype_code)
result = execute_formula(compiled_code)

print(compiled_code)
print("====================================\nResult:")
print(result)
