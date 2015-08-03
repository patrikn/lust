use std::io::{Stdin, stdin, Read};
use std::io::Error;
use std::str::{from_utf8};
use lust::lisp::read::{repl};
extern crate lust;

fn main() {
    let mut buf: &mut Vec<u8> = &mut &mut vec![];
    let bytes = stdin().bytes();
    let mut chars = bytes.scan(buf, scanner);
    repl(&mut chars);
}

fn scanner(buf: &mut &mut Vec<u8>, b: Result<u8, Error>) -> Option<Result<char, Error>> {
    match b {
        Ok(c) => {
            buf.push(c);
            let res =
                match from_utf8(buf) {
                    Ok(s) => Some(Ok(s.chars().next().expect("Non-empty UTF-8 yielded empty string"))),
                    Err(e) => None
                };
            if res.is_some() {
                buf.clear();
            }
            res
        }
        Err(e) => Some(Err(e))
    }
}
