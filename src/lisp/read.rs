use std::iter::{Iterator,Peekable};
use std::io;
use std::num;
use std::fmt;
use std::error::{Error};
pub use lisp::expr::{Add,Expression,Function,Call,Literal,If,Environment};

#[derive(Debug)]
pub enum ReadError {
    Io(io::Error),
    Invalid(String),
    Parse(num::ParseIntError),
    Eof
}

impl From<io::Error> for ReadError {
    fn from(err: io::Error) -> ReadError {
        ReadError::Io(err)
    }
}

impl From<num::ParseIntError> for ReadError {
    fn from(err: num::ParseIntError) -> ReadError {
        ReadError::Parse(err)
    }
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ReadError::Io(ref err) => write!(f, "IO error: {}", err),
            ReadError::Parse(ref err) => write!(f, "Parse error: {}", err),
            ReadError::Invalid(ref err) => write!(f, "Invalid input: {}", err),
            ReadError::Eof => write!(f, "End of file")
        }
    }
}


pub fn repl(input: &mut Iterator<Item = Result<char, io::Error>>) {
    let peekable = &mut input.peekable();
    let env = Environment::new();
    loop {
        let expr = read_expr(peekable);
        match expr {
            Ok(expr) => match expr.eval(&env) {
                Ok(val) => println!("{}", val),
                Err(e) => println!("Error: {}", e)
            },
            Err(ReadError::Eof) => return,
            Err(e) => println!("Error: {}", e)
        }
    }
}

pub fn read_expr(input: &mut Peekable<&mut Iterator<Item = Result<char, io::Error>>>)
    -> Result<Box<Expression>, ReadError>
{
    let c = match input.peek() {
        Some(&Ok(ref c)) => Some(*c),
        Some(&Err(_)) => None,
        None => return Err(ReadError::Eof)
    };
    match c {
        Some(c) => match c {
            '(' => {input.next();
                    return Ok(Box::new(Call::new(try!(read_function_name(input)),
                                                  try!(read_function_params(input)))))
                   },
            '0'...'9'|'+'|'-' => Ok(Box::new(Literal::new(try!(read_number(input))))),
            ' '|'\n'|'\r' => {input.next(); Ok(try!(read_expr(input))) },
            _ => { input.next(); Err(ReadError::Invalid(format!("Invalid input '{}'", c))) }
        },
        None => return Err(From::from(input.next().expect("Input disappeared!").err().expect("Error disappeared!")))
    }
}

pub fn read_function_name(input: &mut Peekable<&mut Iterator<Item = Result<char, io::Error>>>) -> Result<Box<Function>, ReadError> {
    let mut name = String::new();
    for c in input {
        match c {
            Ok(' ') => break,
            Ok(c) => name.push(c),
            Err(e) => return Err(From::from(e))
        }
    }

    let n: &str = &name;
    match n {
        "+" => return Ok(Box::new(Add::new())),
        "if" => return Ok(Box::new(If::new())),
        _ => Err(ReadError::Invalid(format!("Unknown function '{}'", name)))
    }
}

macro_rules! try_peek {
    ($expr:expr) => ({{let stupid_rust = {
                          let peek = $expr.peek();
                          match peek {
                              Some(&Result::Ok(ref val)) => Some(*val),
                              Some(&Result::Err(_)) => None,
                              None => None
                          }
                      };
                      match stupid_rust {
                          Some(c) => Some(c),
                          None => {
                              match $expr.next() {
                                  Some(Err(e)) => return Err(From::from(e)),
                                  None => None,
                                  Some(Ok(_)) => panic!("peek and next disagree")
                              }
                          }
                      }
                      }}
                    )
}

pub fn read_function_params(input: &mut Peekable<&mut Iterator<Item = Result<char, io::Error>>>) -> Result<Vec<Box<Expression>>, ReadError> {
    let mut params: Vec<Box<Expression>> = vec![];
    let mut acc = String::new();
    loop {
        let c = match try_peek!(input) {
            Some(c) => c,
            None => break
        };
        acc.push(c);
        println!("Reading param starting with {}", c);
        match c {
            '0'...'9'|'-' => params.push(Box::new(Literal::new(try!(read_number(input))))),
            '(' => params.push(try!(read_expr(input))),
            ' ' => { input.next(); continue },
            ')' => { input.next(); return Ok(params) },
            _ => { input.next(); return Err(ReadError::Invalid(format!("Invalid input '{}'", c))) }
        }
    }
    Err(ReadError::Eof)
}

pub fn read_number(input: &mut Peekable<&mut Iterator<Item = Result<char, io::Error>>>) -> Result<i64, ReadError> {
    let mut buf = String::new();
    loop {
        let c = try_peek!(input);
        match c {
            Some(c @ '-')       => { buf.push(c); input.next(); if buf.len() > 1 { return Err(ReadError::Invalid(format!("invalid number {}", buf))); } },
            Some(c @ '0'...'9') => { buf.push(c); input.next(); },
            Some(' ')       => break,
            Some(')')       => break,
            None            => { input.next(); return Err(ReadError::Eof) },
            Some(c)         => { input.next(); return Err(ReadError::Invalid(format!("Invalid input '{}'", c))) }
        }
    }
    Ok(try!(buf.parse()))
}


#[cfg(test)]
mod test {
    use super::*;
    use std::iter::{Iterator,Map};
    use std::str::{Chars};
    use std::io::Error;

    fn char_to_result(c: char) -> Result<char, Error> {
        Ok(c)
    }

    fn input(s: &'static str) -> Map<Chars<'static>, fn(char)->Result<char, Error>> {
        return s.chars().map(char_to_result);
    }

    fn iterator<'a, T>(iterator: &'a mut Iterator<Item=T>) -> &'a mut Iterator<Item=T> {
        iterator
    }

    #[test]
    fn test_read_add_function() {
        let mut m = input("+");
        let peekable = &mut iterator(&mut m).peekable();
        match read_function_name(peekable) {
            Err(_) => panic!("Didn't get function"),
            _ => ()
        };
    }

    #[test]
    fn test_read_unknown_function() {
        let mut m = input("apa");
        let peekable = &mut iterator(&mut m).peekable();
        match read_function_name(peekable) {
            Ok(_) => panic!("Should get error"),
            _ => ()
        }
    }

    #[test]
    fn test_read_number() {
        let mut m = input("14 ");
        let peekable = &mut iterator(&mut m).peekable();
        let val = read_number(peekable).unwrap();

        assert_eq!(14, val);
    }

    #[test]
    fn test_read_negative_number() {
        let mut m = input("-14 ");
        let peekable = &mut iterator(&mut m).peekable();
        let val = read_number(peekable).unwrap();

        assert_eq!(-14, val);
    }

    #[test]
    fn test_read_number_right_paren() {
        let mut m = input("2701)");
        let peekable = &mut iterator(&mut m).peekable();
        let val = read_number(peekable).unwrap();

        assert_eq!(2701, val);

        let next = peekable.next().expect("Right paren was consumed");
        assert_eq!(')', next.unwrap());
    }

    #[test]
    fn test_read_number_params() {
        let env = Environment::new();
        let mut m = input("1 2)");
        let peekable = &mut iterator(&mut m).peekable();
        let params = read_function_params(peekable).unwrap();
        assert_eq!(2, params.len());
        assert_eq!(1, params[0].eval(&env).unwrap());
        assert_eq!(2, params[1].eval(&env).unwrap());
    }

    #[test]
    fn test_read_expr() {
        let env = Environment::new();
        let mut m = input("(+ 1 2)");
        let peekable = &mut iterator(&mut m).peekable();

        let expr = read_expr(peekable).unwrap();
        assert_eq!(3, expr.eval(&env).unwrap());
    }

    #[test]
    fn test_read_nested_expr() {
        let env = Environment::new();
        let mut m = input("(+ 1 (+ 1 1))");
        let peekable = &mut iterator(&mut m).peekable();
        let expr = read_expr(peekable);
        assert_eq!(3, expr.unwrap().eval(&env).unwrap());
    }

    #[test]
    fn test_read_if_nonzero() {
        let env = Environment::new();
        let mut m = input("(if (+ 1 1) 1 2)");
        let peekable = &mut iterator(&mut m).peekable();
        let expr = read_expr(peekable).unwrap();
        assert_eq!(1, expr.eval(&env).unwrap());
    }

    #[test]
    fn test_read_if_zero() {
        let env = Environment::new();
        let mut m = input("(if (+ 1 -1) 1 (+ 2 3))");
        let peekable = &mut iterator(&mut m).peekable();
        let expr = read_expr(peekable).unwrap();
        assert_eq!(5, expr.eval(&env).unwrap());
    }
}
