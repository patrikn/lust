use std::iter::{Iterator,Peekable};
use std::error::{Error};
pub use lisp::expr::{Add,Expression,Function,Call,Literal};

pub fn repl<E: Error>(input: &mut Iterator<Item = Result<char, E>>) {
    loop {
        let filtered: &mut Iterator<Item = char> = &mut input.map(|c| -> char {read_char(c)});
        let peekable: &mut Peekable<&mut Iterator<Item = char>> = &mut filtered.peekable();
        let expr = read_expr(peekable);
        let val = expr.eval();
        println!("{}", val);
    }
}

pub fn read_char<E: Error>(c: Result<char, E>) -> char {
    c.ok().expect("IO error")
}

pub fn read_expr(input: &mut Peekable<&mut Iterator<Item = char>>)
    -> Box<Expression>
{
    let c = *input.peek().expect("EOF while reading expr");
    match c {
        '(' => {input.next();
                return Box::new(Call::new(read_function_name(input),
                                          read_function_params(input)))
               },
        '0'...'9'|'+'|'-' => Box::new(Literal::new(read_number(input))),
        ' '|'\n'|'\r' => {input.next(); read_expr(input) },
        _ => panic!("Invalid input '{}'", c)
    }
}

pub fn read_function_name(input: &mut Peekable<&mut Iterator<Item = char>>) -> Box<Function> {
    let mut name = String::new();
    for c in input {
        match c {
            ' ' => break,
            _ => name.push(c)
        }
    }

    let n: &str = &name;
    match n {
        "+" => return Box::new(Add::new()),
        _ => panic!("Unknown function '{}'", name)
    }
}

pub fn read_function_params(input: &mut Peekable<&mut Iterator<Item = char>>) -> Vec<Box<Expression>> {
    let mut params: Vec<Box<Expression>> = vec![];
    loop {
        let c = *input.peek().expect("EOF while reading params");
        println!("Reading param starting with {}", c);
        match c {
            '0'...'9'|'-' => params.push(Box::new(Literal::new(read_number(input)))),
            '(' => params.push(read_expr(input)),
            ' ' => { input.next(); continue },
            ')' => { input.next(); return params },
            _ => panic!("Invalid input '{}'", c)
        }
    }
    panic!("EOF while reading params");
}

pub fn read_number(input: &mut Peekable<&mut Iterator<Item = char>>) -> i64 {
    let mut buf = String::new();
    loop {
        let c = *input.peek().expect("EOF");
        match c {
            '-'       => { buf.push(c); input.next(); if buf.len() > 1 { panic!("invalid number {}", buf); } },
            '0'...'9' => { buf.push(c); input.next(); },
            ' '       => break,
            ')'       => break,
            _         => panic!("Invalid input '{}'", c)
        }
    }
    match buf.parse() {
        Ok(val) => return val,
        Err(err) => { panic!("Not a number: {}", err) }
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use std::iter::{Iterator,Peekable};
    use std::str::{Chars};

    #[test]
    fn test_read_add_function() {
        // Not sure how to tell what I get back but at least it doesn't panic
        let foo: &mut Iterator<Item = char> = &mut "+".chars();
        let peekable: &mut Peekable<&mut Iterator<Item = char>> = &mut foo.peekable();
        read_function_name(peekable);
    }

    #[test]
    #[should_panic(expected="Unknown function 'apa'")]
    fn test_read_unknown_function() {
        // Not sure how to tell what I get back but at least it doesn't panic
        let foo: &mut Iterator<Item = char> = &mut "apa".chars();
        let peekable: &mut Peekable<&mut Iterator<Item = char>> = &mut foo.peekable();
        read_function_name(peekable);
    }

    #[test]
    fn test_read_number() {
        let foo: &mut Iterator<Item = char> = &mut "14 ".chars();
        let peekable: &mut Peekable<&mut Iterator<Item = char>> = &mut foo.peekable();
        let val = read_number(peekable);

        assert_eq!(14, val);
    }

    #[test]
    fn test_read_negative_number() {
        let foo: &mut Iterator<Item = char> = &mut "-14 ".chars();
        let peekable: &mut Peekable<&mut Iterator<Item = char>> = &mut foo.peekable();
        let val = read_number(peekable);

        assert_eq!(-14, val);
    }

    #[test]
    fn test_read_number_right_paren() {
        let foo: &mut Iterator<Item = char> = &mut "2701)".chars();
        let peekable: &mut Peekable<&mut Iterator<Item = char>> = &mut foo.peekable();
        let val = read_number(peekable);

        assert_eq!(2701, val);
    }

    #[test]
    fn test_read_number_params() {
        let foo: &mut Iterator<Item = char> = &mut "1 2)".chars();
        let peekable: &mut Peekable<&mut Iterator<Item = char>> = &mut foo.peekable();
        let params = read_function_params(peekable);
        assert_eq!(2, params.len());
        assert_eq!(1, params[0].eval());
        assert_eq!(2, params[1].eval());
    }

    #[test]
    fn test_read_expr() {
        let foo: &mut Iterator<Item = char> = &mut "(+ 1 2)".chars();
        let peekable: &mut Peekable<&mut Iterator<Item = char>> = &mut foo.peekable();
        let expr = read_expr(peekable);
        assert_eq!(3, expr.eval());
    }

    #[test]
    fn test_read_nested_expr() {
        let foo: &mut Iterator<Item = char> = &mut "(+ 1 (+ 1 1))".chars();
        let peekable: &mut Peekable<&mut Iterator<Item = char>> = &mut foo.peekable();
        let expr = read_expr(peekable);
        assert_eq!(3, expr.eval());
    }
}
