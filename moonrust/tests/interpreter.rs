#[cfg(test)]
mod tests {
    use moonrust::interpreter::environment;
    use moonrust::interpreter::ASTExecError;
    use moonrust::interpreter::LuaValue;
    use moonrust::parser::ASTParseError;
    use moonrust::AST;
    use std::cell::RefCell;
    use std::fs;
    use std::process;
    use std::rc::Rc;

    fn parse_file(file: &str) -> Result<AST, ASTParseError> {
        // Read file
        let src: String = match fs::read_to_string(file) {
            Ok(src) => src,
            Err(err) => {
                eprintln!("File read error [{file}; {err}]");
                process::exit(1);
            }
        };

        src.parse::<moonrust::AST>()
    }

    fn run_ast(ast: AST, buffer: Rc<RefCell<Vec<String>>>) -> Result<(), ASTExecError> {
        // Execute the program
        // Initialize environment
        let mut env = environment::Env::new();
        // Insert test print function to environment
        env.insert_global(
            "print".to_string(),
            LuaValue::new(moonrust::interpreter::LuaVal::TestPrint(buffer)),
        );
        ast.exec(&mut env)
    }

    fn test_interpreter(src: &str, expected_output: &str) {
        let buffer: Vec<String> = vec![];
        let buffer = Rc::new(RefCell::new(buffer));

        let ast = parse_file(src).unwrap();
        run_ast(ast, Rc::clone(&buffer)).unwrap();
        assert_eq!(expected_output, buffer.borrow().join("\n"));
    }

    fn test_interpreter_error(src: &str, error_message: &str) {
        let buffer: Vec<String> = vec![];
        let buffer = Rc::new(RefCell::new(buffer));

        let ast = parse_file(src).unwrap();
        // Assert error
        assert_eq!(
            run_ast(ast, Rc::clone(&buffer)),
            Err(ASTExecError::new(error_message))
        );
    }

    #[test]
    fn test_simple_lua() {
        let src = "assets/simple.lua";
        let expected_output = "18.0\nHello, world!";
        test_interpreter(src, expected_output);
    }

    #[test]
    fn test_arguments_lua() {
        let expected_output = "1 1 2 3 nil nil\n1";
        let src = "assets/arguments.lua";
        test_interpreter(src, expected_output);
    }

    #[test]
    fn test_closure1_lua() {
        let expected_output = "true\ntrue\ntrue";
        let src = "assets/closure1.lua";
        test_interpreter(src, expected_output);
    }

    // TODO: Not working right now
    // #[test]
    // fn test_donut_lua() {

    // }

    #[test]
    fn test_factorial_lua() {
        let expected_output = "2432902008176640000";
        let src = "assets/factorial.lua";
        test_interpreter(src, expected_output);
    }

    #[test]
    fn test_for_break_lua() {
        let expected_output = "1\n2\n3\n4\nLoop ended";
        let src = "assets/for_break.lua";
        test_interpreter(src, expected_output);
    }

    #[test]
    fn test_for_numeral_lua() {
        let expected_output = "1\n2\n3\n4\nLoop ended";
        let src = "assets/for_break.lua";
        test_interpreter(src, expected_output);
    }

    #[test]
    fn test_for_return_lua() {
        let expected_output = "3";
        let src = "assets/for_return.lua";
        test_interpreter(src, expected_output);
    }

    #[test]
    fn test_function_def1_lua() {
        let expected_output = "5\n25";
        let src = "assets/function_def1.lua";
        test_interpreter(src, expected_output);
    }

    #[test]
    fn test_function_def2_lua() {
        let expected_output = "Hello, World!\nhello\n\nworld\n2 1\n3";
        let src = "assets/function_def2.lua";
        test_interpreter(src, expected_output);
    }

    #[test]
    fn test_function_env_lua() {
        let expected_output = "1";
        let src = "assets/function_env.lua";
        test_interpreter(src, expected_output);
    }

    #[test]
    fn test_function_env2_lua() {
        let expected_output = "nil\n2\n1 nil\n1 3";
        let src = "assets/function_env2.lua";
        test_interpreter(src, expected_output);
    }

    #[test]
    fn test_function_lua() {
        let expected_output = "5";
        let src = "assets/function.lua";
        test_interpreter(src, expected_output);
    }

    #[test]
    fn test_if_else_lua() {
        let expected_output = "x is nil\nx is false\nx is truthy\nx is greater than or equal to 10\nx is between 0 and 10";
        let src = "assets/if_else.lua";
        test_interpreter(src, expected_output);
    }

    #[test]
    fn test_local_func1_lua() {
        let expected_output = "true\ntrue\ntrue\ntrue";
        let src = "assets/local_func1.lua";
        test_interpreter(src, expected_output);
    }

    #[test]
    fn test_local_func2_lua() {
        let expected_output = "\ntrue\ntrue\ntrue\n1";
        let src = "assets/local_func2.lua";
        test_interpreter(src, expected_output);
    }

    #[test]
    fn test_memory_error_lua() {
        let expected_error = "Cannot execute opration on values that are not numbers";
        let src = "assets/memory_error.lua";
        test_interpreter_error(src, expected_error);
    }

    #[test]
    fn test_multiple_return_lua() {
        let expected_output = "1 1 1 1 2 3";
        let src = "assets/multiple_return.lua";
        test_interpreter(src, expected_output);
    }

    #[test]
    fn test_nested_for_return_lua() {
        let expected_output = "1 4";
        let src = "assets/nested_for_return.lua";
        test_interpreter(src, expected_output);
    }

    #[test]
    fn test_nested_for_lua() {
        let expected_output =
            "1 1\n1 2\n1 3\n1 4\n1 5\n1 6\n2 1\n2 2\n2 3\n2 4\n3 1\n3 2\nLoop ended";
        let src = "assets/nested_for.lua";
        test_interpreter(src, expected_output);
    }

    // object.lua
    // Self is not implemented
    #[test]
    fn test_object_lua() {
        let expected_output = "Position: 10 20";
        let src = "assets/object.lua";
        test_interpreter(src, expected_output);
    }

    #[test]
    fn test_prob1_lua() {
        let expected_output = "2\n1";
        let src = "assets/prob1.lua";
        test_interpreter(src, expected_output);
    }

    #[test]
    fn test_prob2_lua() {
        let expected_output = "3\n2";
        let src = "assets/prob2.lua";
        test_interpreter(src, expected_output);
    }

    #[test]
    fn test_prob3_lua() {
        let expected_output = "1\nnil";
        let src = "assets/prob3.lua";
        test_interpreter(src, expected_output);
    }

    #[test]
    fn test_prob4_lua() {
        let expected_output = "5\n5";
        let src = "assets/prob4.lua";
        test_interpreter(src, expected_output);
    }

    #[test]
    fn test_prob5_lua() {
        let expected_output = "3";
        let src = "assets/prob5.lua";
        test_interpreter(src, expected_output);
    }

    #[test]
    fn test_repeat_break_lua() {
        let expected_output = "1\n2\n3\n4\nLoop ended";
        let src = "assets/repeat_break.lua";
        test_interpreter(src, expected_output);
    }

    #[test]
    fn test_repeat_return_lua() {
        let expected_output = "3";
        let src = "assets/repeat_return.lua";
        test_interpreter(src, expected_output);
    }

    #[test]
    fn test_table1_lua() {
        let expected_output = "x y 23 45 1\nx y 23 45 1\nz y 23 45 1\nz y 23 45 1";
        let src = "assets/table1.lua";
        test_interpreter(src, expected_output);
    }

    #[test]
    fn test_table2_lua() {
        let expected_output = "x y 23 10 45\nx y 23 10 45\nz y 23 10 45\nz y 23 10 45";
        let src = "assets/table2.lua";
        test_interpreter(src, expected_output);
    }

    #[test]
    fn test_while_break_lua() {
        let expected_output = "1\n2\n2\n3\n3\n4\n4\n5\nLoop ended";
        let src = "assets/while_break.lua";
        test_interpreter(src, expected_output);
    }

    #[test]
    fn test_while_return_lua() {
        let expected_output = "3";
        let src = "assets/while_return.lua";
        test_interpreter(src, expected_output);
    }

    #[test]
    fn test_print_rows() {
        let expected_output = "*\n**\n***\n****\n*****";
        let src = "assets/print_rows.lua";
        test_interpreter(src, expected_output);
    }

    #[test]
    fn test_fibonacci() {
        let expected_output = "0\n1\n1\n2\n3\n5\n8\n13\n21\n34\n55";
        let src = "assets/fibonacci_fixed.lua";
        test_interpreter(src, expected_output);
    }

    // TODO: add more test cases
}
