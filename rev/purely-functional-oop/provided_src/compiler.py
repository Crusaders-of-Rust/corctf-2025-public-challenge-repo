import re
from lark import Lark

DEBUG = False
def debug_print(*args, **kwargs):
    if DEBUG:
        print(*args, **kwargs)

# compiles the oop language into a google sheets formula
lang_grammar = """
    start: statement+

    statement: class_definition
        | function_definition
        | class_constructor
        | instruction
    
    instruction: "let" "this" "." VARNAME "=" expression ";" -> class_assignment
        | "let" VARNAME "=" expression ";"          -> assignment
        | "return" expression ";"                   -> return
    
    // an expression can be a a term (variable or literal),
    // a ternary expression, a method call, a function call,
    // or an expression can also be grouped with additional parentheses

    // Highest-level expression rule
    ?expression: ternary_expression

    // Lowest precedence (wrapping all other expressions)
    ?ternary_expression: expression "?" expression ":" expression -> ternary_expression
                        | member_access

    // Field/method access (high precedence, recursive)
    ?member_access: member_access "." invocation    -> method_call
                | "this" "." VARNAME                -> field_access
                | primary

    // Primary expressions (highest precedence, only these can represent the callee of a method call)
    ?primary: literal                               -> literal_expression
            | "new" VARNAME "(" argument_list ")"   -> class_instantiation
            | invocation                            -> function_call
            | VARNAME                               -> variable_access
            | "(" expression ")"                    -> nested_expression

    invocation: VARNAME "(" argument_list ")"
    argument_list: (expression ("," expression)*)?
    
    literal: INT                        -> number_literal
        | FLOAT                         -> number_literal
        | STRING                        -> string_literal
        | "true"                        -> true_literal
        | "false"                       -> false_literal
    
    function_definition: "fn" VARNAME "(" argument_list ")" code_block
    class_constructor: "constructor" "(" argument_list ")" code_block

    // a code block cannot declare a class or function within it
    code_block: "{" statement* "}"
    
    // a class can only declare a constructor and methods
    // but all statements will get parsed, for a clearer error message
    class_definition: "class" VARNAME "{" statement* "}"

    %import common.CNAME -> VARNAME
    %import common.SIGNED_INT -> INT
    %import common.SIGNED_FLOAT -> FLOAT
    %import common.ESCAPED_STRING -> STRING
    %import common.WS -> WHITESPACE
    %ignore WHITESPACE
"""

parser = Lark(lang_grammar)


INDENT_STRING = "  "
def debug_print_parse_tree(tree, indent=0): # only prints if DEBUG is True
    debug_print(INDENT_STRING * indent + str(tree.data))
    for child in tree.children:
        if isinstance(child, str):
            debug_print(INDENT_STRING * (indent + 1) + child)
        else:
            debug_print_parse_tree(child, indent + 1)


RESERVED_KEYWORDS = { "class", "fn", "constructor", "let", "return", "new", "this", "true", "false", "_f", "_message", "_constructor", "_message_match", "_raise_error_internal" }
def ensure_valid_name(variable_name, allow_this=False):
    # make sure the variable name is not a spreadsheet cell reference (e.g. "A1", "AA12", etc.)
    # using regex to elimiate these invalid names of [a-zA-Z]+[0-9]+, which is the valid identifier pattern
    if re.match(r"^[a-zA-Z]+[0-9]+$", variable_name):
        raise SyntaxError(f"Invalid symbol name matches cell ID, try adding an underscore: {variable_name}")
    if variable_name == "this":
        if not allow_this:
            raise SyntaxError("The 'this' keyword can only be used in class definitions")
    elif variable_name in RESERVED_KEYWORDS:
        raise SyntaxError(f"Variable name matches reserved keyword: {variable_name}")
    return variable_name

def ensure_statement_node(statement_node):
    if statement_node.data != "statement":
        raise SyntaxError(f"Expected 'statement', got {statement_node.data}")
    return statement_node


def parse_root_node(root_node):
    if root_node.data != "start":
        raise SyntaxError(f"Expected 'start', got {root_node.data}")
    
    program_code = ""
    for statement_node in root_node.children:
        code, returned = parse_outer_statement_node(statement_node)
        program_code += code
        if returned:
            break
    else:
        raise SyntaxError("Program must return a value")

    compiled_result = f"""
    =LET(
        _raise_error_internal, LAMBDA(LAMBDA(a, a)("illegal", "syntax")),
        _message_match, LAMBDA(expected, LAMBDA(actual,
            IFERROR((actual = expected), FALSE)
        )),

        make_builtin_bootstrap, LAMBDA(f, LAMBDA(raw,
            LAMBDA(_message,
                IF(_message_match("_rawVal")(_message),
                    raw,
                IF(_message_match("plus")(_message),
                    LAMBDA(rhs, f(f)(raw + rhs("_rawVal"))),
                IF(_message_match("minus")(_message),
                    LAMBDA(rhs, f(f)(raw - rhs("_rawVal"))),
                IF(_message_match("times")(_message),
                    LAMBDA(rhs, f(f)(raw * rhs("_rawVal"))),
                IF(_message_match("divide")(_message),
                    LAMBDA(rhs, f(f)(raw / rhs("_rawVal"))),
                IF(_message_match("equals")(_message),
                    LAMBDA(rhs, f(f)(raw = rhs("_rawVal"))), 
                IF(_message_match("notEquals")(_message),
                    LAMBDA(rhs, f(f)(raw <> rhs("_rawVal"))), 
                IF(_message_match("and")(_message),
                    LAMBDA(rhs, f(f)(AND(raw, rhs("_rawVal")))),
                IF(_message_match("or")(_message),
                    LAMBDA(rhs, f(f)(OR(raw, rhs("_rawVal")))),
                IF(_message_match("xor")(_message),
                    LAMBDA(rhs, f(f)(XOR(raw, rhs("_rawVal")))),
                IF(_message_match("lessThan")(_message),
                    LAMBDA(rhs, f(f)(raw < rhs("_rawVal"))),
                IF(_message_match("lessThanOrEquals")(_message),
                    LAMBDA(rhs, f(f)(raw <= rhs("_rawVal"))),
                IF(_message_match("greaterThan")(_message),
                    LAMBDA(rhs, f(f)(raw > rhs("_rawVal"))),
                IF(_message_match("greaterThanOrEquals")(_message),
                    LAMBDA(rhs, f(f)(raw <= rhs("_rawVal"))),
                IF(_message_match("negate")(_message),
                    f(f)(NOT(raw)),
                IF(_message_match("factorial")(_message),
                    f(f)(FACT(raw)),
                IF(_message_match("log")(_message),
                    f(f)(LOG(raw)),
                IF(_message_match("concat")(_message),
                    LAMBDA(rhs, f(f)(CONCAT(raw, rhs("_rawVal")))),
                _raise_error_internal() 
                ))))))))))))))))))
            )
        )),

        make_builtin, (make_builtin_bootstrap) (make_builtin_bootstrap),

        {program_code}("_rawVal")
    )
    """
    return compiled_result

def parse_outer_statement_node(statement_node):
    ensure_statement_node(statement_node)

    statement = statement_node.children[0]
    statement_type = statement.data

    if statement_type == "class_definition":
        # The class name is already in the scope because it was preprocessed
        class_name = statement.children[0]
        ensure_valid_name(class_name)
        debug_print(f"Class definition: {class_name}")
        debug_print(f"Class body: {statement.children[1:]}")

        constructor_statement = statement.children[1]  # the constructor is always the second child
        constructor = constructor_statement.children[0]
        if constructor.data != "class_constructor":
            raise SyntaxError(f"Expected 'class_constructor' as first statement in class, got {constructor.data}")
        (constructor_opening, constructor_closing, constructor_args, constructor_args_comma), returned = parse_inner_statement_node(constructor_statement, scope='class', outer_name=class_name)
        if returned:
            raise SyntaxError("Class constructors cannot return values")
        
        # parse the rest of the class body
        message_handler = "_raise_error_internal()" # default value will throw an error if evaluated
        for inner_statement_node in statement.children[2:]:
            (opening, closing), returned = parse_inner_statement_node(inner_statement_node, scope='class', outer_name=class_name)
            if returned:
                raise SyntaxError("Unexpected return statement in class body")
            message_handler = f"{opening}{message_handler}{closing}"

        # template for a class definition
        compiled_result = f"""
        class_{class_name}, LAMBDA(_constructor, LAMBDA(this,
            {constructor_opening}
                LAMBDA(_message,
                    {message_handler}
                )
            {constructor_closing}
        )),
        bootstrap_new_{class_name}, LAMBDA(_f, LAMBDA({constructor_args_comma} class_{class_name}(LAMBDA({constructor_args_comma} (LAMBDA(_message, _f(_f)({constructor_args})(_message))))) (LAMBDA(_message, _f(_f)({constructor_args})(_message))) ({constructor_args}))),
        new_{class_name}, (bootstrap_new_{class_name}) (bootstrap_new_{class_name}),
        """
        return (compiled_result, False)
    elif statement_type == "function_definition":
        # Since this is an outer statement, this function is static
        return parse_static_function(statement)
    elif statement_type == "class_constructor":
        # Class constructors cannot be defined at the outer (static) scope
        raise SyntaxError("Class constructors must be defined within a class")
    elif statement_type == "class_assignment":
        # Instance variables cannot be assigned at the outer (static) scope
        raise SyntaxError("Instance variables cannot be assigned at the outer (static) scope")
    elif statement_type == "assignment":
        # Static variable assignment (normal behavior; delegate)
        return parse_inner_statement_node(statement_node, scope='static', outer_name=None)
    elif statement_type == "return":
        # Program return value (normal behavior; delegate)
        return parse_inner_statement_node(statement_node, scope='static', outer_name=None)
    else:
        raise SyntaxError(f"Unknown statement type: {statement_type}")

def parse_static_function(function_declaration_statement):
    if function_declaration_statement.data != "function_definition":
        raise SyntaxError(f"Expected 'function_definition', got {function_declaration_statement.data}")

    function_name = function_declaration_statement.children[0]
    argument_list = function_declaration_statement.children[1]
    code_block = function_declaration_statement.children[2]
    ensure_valid_name(function_name)
    debug_print(f"Parsing static function: {function_name}")
    args = ", ".join(parse_expression(arg, 'static', function_name) for arg in argument_list.children)
    if not args:
        args_comma = ""
    else:
        args_comma = f"{args},"

    # parse the function body
    function_code = ""
    for statement_node in code_block.children:
        code, returned = parse_inner_statement_node(statement_node, scope='static', outer_name=function_name)
        function_code += code
        if returned:
            break
    else:
        raise SyntaxError("Function must return a value")

    compiled_result = f"""
    bootstrap_{function_name}, LAMBDA(_f,
        LAMBDA({args_comma}
            LET({function_code})
        )
    ),
    static_function_{function_name}, (bootstrap_{function_name}) (bootstrap_{function_name}),
    """

    return (compiled_result, False)

# outer_name is the name of the enclosing class or static function, for recursive calls
# scope is either 'static', 'class', or 'constructor'
def parse_inner_statement_node(statement_node, scope, outer_name):
    ensure_statement_node(statement_node)

    statement = statement_node.children[0]
    statement_type = statement.data

    if statement_type == "class_definition":
        # Not allowed in an inner parse
        raise SyntaxError("Class definitions cannot be nested within other statements")
    elif statement_type == "function_definition":
        # Functions can only be nested for a instance method
        if scope != 'class':
            raise SyntaxError("Instance methods can only be defined within a class")
        if outer_name is None:
            raise SyntaxError("Outer name must be provided for function definition")
        function_name = statement.children[0]
        argument_list = statement.children[1]
        code_block = statement.children[2]
        ensure_valid_name(function_name)

        debug_print(f"Parsing instance method: {function_name} in class {outer_name}")
        args = ", ".join(parse_expression(arg, scope, outer_name) for arg in argument_list.children)

        # parse the function body
        function_code = ""
        for statement_node in code_block.children:
            code, returned = parse_inner_statement_node(statement_node, scope=scope, outer_name=outer_name) # keep the class name as outer_name
            function_code += code
            if returned:
                break
        else:
            raise SyntaxError("Function must return a value")
        if not args:
            # no curried arguments
            opening = f"""
            IF(_message_match("{function_name}")(_message),
                LET({function_code}),
            """
            closing = ")"
        else:
            opening = f"""
            IF(_message_match("{function_name}")(_message),
                LAMBDA({args}, 
                    LET({function_code})
                ),
            """
            closing = f")"
        return ((opening, closing), False)
    elif statement_type == "class_constructor":
        if scope != 'class':
            raise SyntaxError("Class constructors can only be defined at the top of a class")
        argument_list = statement.children[0]
        code_block = statement.children[1]

        args = ", ".join(parse_expression(arg, scope, outer_name) for arg in argument_list.children)
        if not args:
            args_comma = ""
        else:
            args_comma = f"{args},"
        constructor_opening = f"LAMBDA({args_comma} LET(\n"
        constructor_closing = "))"
        for inner_statement_node in code_block.children:
            inner_statement_code, returned = parse_inner_statement_node(inner_statement_node, scope='constructor', outer_name=outer_name)
            constructor_opening += inner_statement_code
            if returned:
                raise SyntaxError("Constructor cannot return a value")
        return ((constructor_opening, constructor_closing, args, args_comma), False)
    elif statement_type == "class_assignment":
        if scope != 'constructor':
            raise SyntaxError("Instance variables can only be assigned within a class constructor")
        if outer_name is None:
            raise SyntaxError("Outer name must be provided for class assignment")
        variable_name = statement.children[0]
        ensure_valid_name(variable_name)
        rhs = parse_expression(statement.children[1], scope, outer_name)
        return (f"this_{variable_name}, {rhs},\n", False)
    elif statement_type == "assignment":
        variable_name = statement.children[0]
        ensure_valid_name(variable_name)
        rhs = parse_expression(statement.children[1], scope, outer_name)
        return (f"{variable_name}, {rhs},\n", False)
    elif statement_type == "return":
        rhs = parse_expression(statement.children[0], scope, outer_name)
        return (f"\n{rhs}\n", True)
    else:
        raise SyntaxError(f"Unknown statement type: {statement_type}")

def parse_expression(expression_node, scope, outer_name):
    expression_type = expression_node.data
    if expression_type == "literal_expression":
        return parse_literal_expression(expression_node.children[0])
    elif expression_type == "class_instantiation":
        class_name = expression_node.children[0]
        argument_list = expression_node.children[1]
        ensure_valid_name(class_name)
        args = ", ".join(parse_expression(arg, scope, outer_name) for arg in argument_list.children)
        debug_print(f"Parsing class instantiation: {class_name} with args: {args} in scope {scope} with outer name {outer_name}")
        if scope != 'static' and class_name == outer_name:
            return f"_constructor({args})" # recursive instantiation using _constructor lambda
        else:
            return f"new_{class_name}({args})" # non-recursive instantiation
    elif expression_type == "field_access":
        if scope == 'static':
            raise SyntaxError("Field access is not allowed at the static scope")
        if outer_name is None:
            raise SyntaxError("Outer name must be provided for field access")
        field_name = expression_node.children[0]
        ensure_valid_name(field_name, allow_this=True)
        return f"this_{field_name}"
    elif expression_type == "method_call":
        variable_access_node = expression_node.children[0]
        invocation_node = expression_node.children[1]
        if invocation_node.data != "invocation":
            raise SyntaxError(f"Unexpected syntax node in method call: {invocation_node.data}")
        callee = parse_expression(variable_access_node, scope, outer_name)
        method_name = invocation_node.children[0]
        argument_list = invocation_node.children[1]
        ensure_valid_name(method_name)

        args = ", ".join(parse_expression(arg, scope, outer_name) for arg in argument_list.children)
        if not args:
            return f"{callee}(\"{method_name}\")" # no curried arguments
        else:
            return f"{callee}(\"{method_name}\")({args})"
    elif expression_type == "function_call":
        invocation_node = expression_node.children[0]
        function_name = invocation_node.children[0]
        argument_list = invocation_node.children[1]
        ensure_valid_name(function_name)
        args = ", ".join(parse_expression(arg, scope, outer_name) for arg in argument_list.children)
        if scope == 'static' and function_name == outer_name:
            return f"_f(_f)({args})" # recursive call
        else:
            return f"static_function_{function_name}({args})" # normal static function call
    elif expression_type == "variable_access":
        variable_name = expression_node.children[0]
        allow_this = scope != 'static'  # 'this' is only allowed in class or constructor scope
        debug_print(f"Parsing variable access: {variable_name} in scope {scope} with outer name {outer_name}")
        ensure_valid_name(variable_name, allow_this=allow_this)
        return variable_name
    elif expression_type == "nested_expression":
        inner_expression = parse_expression(expression_node.children[0], scope, outer_name)
        return f"({inner_expression})"
    elif expression_type == "ternary_expression":
        condition = parse_expression(expression_node.children[0], scope, outer_name)
        true_branch = parse_expression(expression_node.children[1], scope, outer_name)
        false_branch = parse_expression(expression_node.children[2], scope, outer_name)
        return f"IF({condition}(\"_rawVal\"), {true_branch}, {false_branch})"
    else:
        raise SyntaxError(f"Unknown expression type: {expression_type}")

def parse_literal_expression(literal_node):
    literal_type = literal_node.data
    if literal_type == "number_literal":
        literal_data = literal_node.children[0]
        return f"make_builtin({literal_data})"
    elif literal_type == "string_literal":
        literal_data = literal_node.children[0]
        return f"make_builtin({literal_data})" # already contains quotes. todo: replace \n and \" with char encoding
    elif literal_type == "true_literal":
        return "make_builtin(TRUE)"
    elif literal_type == "false_literal":
        return "make_builtin(FALSE)"
    else:
        raise SyntaxError(f"Unknown literal type: {literal_type}")

# compiles source code into the executable google sheets formula
def compile_code(source_code):
    parse_tree = parser.parse(source_code)
    debug_print_parse_tree(parse_tree)
    debug_print("Compiling the code...")
    compiled_code = parse_root_node(parse_tree).strip()
    debug_print("Compiled code:")
    debug_print("==========================================")
    debug_print("\n\n")
    debug_print(compiled_code)
    debug_print("\n\n")
    debug_print("==========================================")
    debug_print("\n\n")
    return compiled_code

