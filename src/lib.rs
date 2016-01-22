#[macro_use]
extern crate nom;

extern crate rdf_traits;

use nom::{IResult, alphanumeric, multispace, Err, ErrorKind, not_line_ending};
use std::str::from_utf8;

pub type MyTriple<'a> = rdf_traits::Triple<Subject<'a>, Predicate<'a>, Object<'a>>;

fn is_absolute_uri_char(c: u8) -> bool{
    c != b'>'
}

named!(absolute_uri<&str>, map_res!(
    take_while!(is_absolute_uri_char),
    from_utf8
));

named!(pub uriref <&str>, chain!(
        char!('<') ~
        uri: absolute_uri ~
        char!('>'),
        ||{
            uri
        }
    )
);

named!(pub comment,
    chain!(
        tag!("#") ~
        not_line_ending? ~
        alt!(eol | eof),
        || { &b""[..] }
    )
);

fn eof(input:&[u8]) -> IResult<&[u8], &[u8]> {
    if input.len() == 0 {
        IResult::Done(input, input)
    } else {
        IResult::Error(Err::Code(ErrorKind::Custom(0)))
    }
}

named!(eol,
       alt!(tag!("\r\n") | tag!("\n") | tag!("\u{2028}") | tag!("\u{2029}")));


named!(name, call!(alphanumeric));

named!(pub named_node <&str>,
    map_res!(preceded!(
        tag!("_:"),
        name
    ), from_utf8)
);

named!(subject<Subject>, alt!(
    uriref => { |res| Subject::AbsoluteUri(res)  } |
    named_node => { |res| Subject::NamedNode(res) }
));

named!(predicate<Predicate>, map!(
    uriref,
    |o| Predicate::AbsoluteUri(o)
));

named!(literal<&str>, chain!(
    char!('"') ~
    d: take_until!([b'"']) ~
    char!('"'),
    ||{ from_utf8(d).unwrap() }
));

named!(pub object<Object>, alt!(
    uriref => { |res| Object::AbsoluteUri(res) } |
    named_node => { |res| Object::NamedNode(res) } |
    literal => { |res| Object::Literal(res) }
));

named!(ws, map!(many1!(
    alt!(
        multispace |
        comment
    )
), |_| &b""[..] ));

named!(pub triple<MyTriple>, chain!(
    ws? ~
    s: subject ~
    ws? ~
    p: predicate ~
    ws? ~
    o: object ~
    ws? ~
    char!('.'),
    ||{ MyTriple::new(s, p, o) }
));

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Subject<'a>{
    AbsoluteUri(&'a str),
    NamedNode(&'a str)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Predicate<'a>{
    AbsoluteUri(& 'a str)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Object<'a>{
    AbsoluteUri(& 'a str),
    NamedNode(& 'a str),
    Literal(& 'a str)
}

#[test]
fn it_works() {
    assert_eq!(
        absolute_uri(b"http://test"),
        IResult::Done(&b""[..],"http://test")
    );
    assert_eq!(
        uriref(b"<http://test>"),
        IResult::Done(&b""[..],"http://test")
    );
    assert_eq!(
        comment(b"#test wie das geht \n"),
        IResult::Done(&b""[..],&b""[..])
    );
    assert_eq!(
        name(b"Der92Name"),
        IResult::Done(&b""[..],&b"Der92Name"[..])
    );
    assert_eq!(
        named_node(b"_:name4Node"),
        IResult::Done(&b""[..],"name4Node")
    );
}

#[test]
fn test_subject(){
    assert_eq!(
        subject(b"_:name4Node"),
        IResult::Done(&b""[..],Subject::NamedNode("name4Node"))
    );
    assert_eq!(
        subject(b"<http://tv-laufach.de/Mitglieder>"),
        IResult::Done(&b""[..],Subject::AbsoluteUri("http://tv-laufach.de/Mitglieder"))
    );
}

#[test]
fn test_predicate(){
    assert_eq!(
        predicate(b"<http://tv-laufach.de/Mitglieder>"),
        IResult::Done(&b""[..],Predicate::AbsoluteUri("http://tv-laufach.de/Mitglieder"))
    );
}

#[test]
fn test_literal(){
    assert_eq!(
        literal(b"\"Hallo Welt\""),
        IResult::Done(&b""[..], "Hallo Welt")
    );
}

#[test]
fn test_object(){
    assert_eq!(
        object(&b"_:Named"[..]),
        IResult::Done(&b""[..],Object::NamedNode("Named"))
    );
    assert_eq!(
        object(&b"<http://tv-laufach.de/Mitglieder>"[..]),
        IResult::Done(&b""[..],Object::AbsoluteUri("http://tv-laufach.de/Mitglieder"))
    );
    assert_eq!(
        object(&b"\"Hallo Welt\""[..]),
        IResult::Done(&b""[..],Object::Literal("Hallo Welt"))
    );
}
#[test]
fn test_triple(){
    assert_eq!(
        triple(&b"<http://www.w3.org/2001/sw/RDFCore/ntriples/> <http://purl.org/dc/elements/1.1/creator> \"Dave Beckett\" ."[..]),
        IResult::Done(&b""[..], MyTriple::new(
            Subject::AbsoluteUri("http://www.w3.org/2001/sw/RDFCore/ntriples/"),
            Predicate::AbsoluteUri("http://purl.org/dc/elements/1.1/creator"),
            Object::Literal("Dave Beckett")
        ))
    );
    assert_eq!(
        triple(&b"<http://www.w3.org/2001/sw/RDFCore/ntriples/>    <http://purl.org/dc/elements/1.1/creator> \"Dave Beckett\" ."[..]),
        IResult::Done(&b""[..], MyTriple::new(
            Subject::AbsoluteUri("http://www.w3.org/2001/sw/RDFCore/ntriples/"),
            Predicate::AbsoluteUri("http://purl.org/dc/elements/1.1/creator"),
            Object::Literal("Dave Beckett")
        ))
    );
    assert_eq!(
        triple(&b"#Das ist ein Kommentar\n <http://www.w3.org/2001/sw/RDFCore/ntriples/>    <http://purl.org/dc/elements/1.1/creator> \"Dave Beckett\" ."[..]),
        IResult::Done(&b""[..], MyTriple::new(
            Subject::AbsoluteUri("http://www.w3.org/2001/sw/RDFCore/ntriples/"),
            Predicate::AbsoluteUri("http://purl.org/dc/elements/1.1/creator"),
            Object::Literal("Dave Beckett")
        ))
    );
}
