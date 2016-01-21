#[macro_use]
extern crate nom;

use nom::{IResult,digit};
use nom::IResult::*;

use std::str;
use std::str::FromStr;

fn is_absolutUriChar(c: u8) -> bool{
    c != b' '
}

named!(absoluteUri, take_while!(is_absolutUriChar));

named!(pub uriref <&[u8]>, chain!(
        char!('<') ~
        uri: absoluteUri ~
        char!('>'),
        ||{
            uri
        }
    )
);

pub enum Subject{
    AbsoluteUri(String),
    NamedNode(String)
}

pub enum Predicate{
    AbsoluteUri(String)
}
pub enum Object{
    AbsoluteUri(String),
    NamedNode(String),
    Literal(String)
}
pub struct Triple{
    pub subject: Subject,
    pub predicate: Predicate,
    pub object: Object
}

#[test]
fn it_works() {
    assert_eq!(
        absoluteUri(b"http://test"),
        IResult::Done(&b""[..],&b"http://test"[..])
    );/*
    assert_eq!(
        uriref(b"<http://test>"),
        IResult::Done(b"",b"http://test".to_string())
    );*/
}
