use std::iter::{Iterator,Peekable};
use std::io;
use std::num;
use std::fmt;
use std::error::{Error};
pub use lisp::expr::{Add,Expression,Function,Call,Literal};

#[derive(Debug)]
pub enum ReadError {
    Io(io::Error),
    Invalid(String),
    Parse(num::ParseIntError),
    Eof
}

fn Invalid(message: &str) -> ReadError {
    ReadError::Invalid(message.to_string())
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
    loop {
        let expr = read_expr(peekable);
        match expr {
            Ok(expr) => println!("{}", expr.eval()),
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
        Some(&Err(ref e)) => None,
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
        _ => Err(ReadError::Invalid(format!("Unknown function '{}'", name)))
    }
}

macro_rules! try_peek {
    ($expr:expr) => ({{let stupid_rust = {
                          let peek = $expr.peek();
                          match peek {
                              Some(&Result::Ok(ref val)) => Some(*val),
                              Some(&Result::Err(ref err)) => None,
                              None => None
                          }
                      };
                      match stupid_rust {
                          Some(c) => Some(c),
                          None => {
                              match $expr.next() {
                                  Some(Err(e)) => return Err(From::from(e)),
                                  None => None,
                                  Some(Ok(v)) => panic!("peek and next disagree")
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
        // let stupid_rust = {
        //     let peek = input.peek();
        //     match peek {
        //         Some(&Err(ref e)) => None,
        //         Some(&Ok(ref c)) => Some(*c),
        //         None => break
        //     }
        // };
        // let c = match stupid_rust {
        //     None => return Err(From::from(input.next().expect("Error disappeared").err().expect("Error disappeared!"))),
        //     Some(c) => c
        // };
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
    use std::iter::{Iterator,Peekable};
    use std::str::{Chars};

    #[test]
    fn test_read_add_function() {
        let foo: &mut Iterator<Item = char> = &mut "+".chars();
        let peekable: &mut Peekable<&mut Iterator<Item = char>> = &mut foo.peekable();
        match read_function_name(peekable) {
            Error(e) => panic!("Didn't get function"),
            _ => ()
        }
    }

    #[test]
    fn test_read_unknown_function() {
        let foo: &mut Iterator<Item = char> = &mut "apa".chars();
        let peekable: &mut Peekable<&mut Iterator<Item = char>> = &mut foo.peekable();
        match read_function_name(peekable) {
            Ok(e) => panic!("Should get error"),
            _ => ()
        }
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
