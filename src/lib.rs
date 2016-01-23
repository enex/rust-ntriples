#[macro_use]
extern crate nom;

extern crate rdf_traits;

use nom::{IResult, alphanumeric, multispace, Err, ErrorKind, not_line_ending};
use std::str::from_utf8;

pub type MyTriple<'a> = rdf_traits::Triple<Subject<'a>, Predicate<'a>, Object<'a>>;

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
    Literal{
        value: & 'a str,
        datatype: & 'a str,
        language: &'a str
    }
}



fn is_absolute_uri_char(c: u8) -> bool{
    c != b'>'
}

named!(absolute_uri<&str>, map_res!(
    take_while!(is_absolute_uri_char),
    from_utf8
));

named!(pub iriref <&str>, chain!(
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

named!(eol, alt!(tag!("\r\n") | tag!("\n") | tag!("\u{2028}") | tag!("\u{2029}")));

named!(name, call!(alphanumeric));

named!(pub named_node <&str>,
    map_res!(preceded!(
        tag!("_:"),
        name
    ), from_utf8)
);

named!(subject<Subject>, alt!(
    iriref => { |res| Subject::AbsoluteUri(res)  } |
    named_node => { |res| Subject::NamedNode(res) }
));

named!(predicate<Predicate>, map!(
    iriref,
    |o| Predicate::AbsoluteUri(o)
));

named!(literal_value<&str>, chain!(
    char!('"') ~
    d: take_until!([b'"']) ~
    char!('"'),
    ||{ from_utf8(d).unwrap() }
));

named!(language_tag<&str>,
    map_res!(preceded!(
        tag!("@"),
        name
    ), from_utf8)
);

named!(literal_type<&str>,
    preceded!(
        tag!("^^"),
        iriref
    )
);

named!(literal<Object>, chain!(
    value: literal_value ~
    add: alt!(
        eof => { |_| ("","") } |
        language_tag => { |res| (res, "") } |
        literal_type => { |res| ("", res) }
    ) ,

    ||{ Object::Literal{
        value:    value,
        datatype: add.1,
        language: add.0
    } }
));

named!(pub object<Object>, alt!(
    iriref => { |res| Object::AbsoluteUri(res) } |
    named_node => { |res| Object::NamedNode(res) } |
    literal => { |res| res }
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

#[test]
fn it_works() {
    assert_eq!(
        absolute_uri(b"http://test"),
        IResult::Done(&b""[..],"http://test")
    );
    assert_eq!(
        iriref(b"<http://test>"),
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
fn test_literal_value(){
    assert_eq!(
        literal_value(b"\"Hallo Welt\""),
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
        IResult::Done(&b""[..],Object::Literal{value:"Hallo Welt", datatype:"", language:""})
    );
    assert_eq!(
        object(&b"\"That Seventies Show\"@en"[..]),
        IResult::Done(&b""[..],Object::Literal{value:"That Seventies Show", datatype:"", language:"en"})
    );
    assert_eq!(
        object(&b"\"That Seventies Show\"^^<http://www.w3.org/2001/XMLSchema#string>"[..]),
        IResult::Done(&b""[..],Object::Literal{value:"That Seventies Show", datatype:"http://www.w3.org/2001/XMLSchema#string", language:""})
    );
}
#[test]
fn test_triple(){
    assert_eq!(
        triple(&b"<http://www.w3.org/2001/sw/RDFCore/ntriples/> <http://purl.org/dc/elements/1.1/creator> \"Dave Beckett\" ."[..]),
        IResult::Done(&b""[..], MyTriple::new(
            Subject::AbsoluteUri("http://www.w3.org/2001/sw/RDFCore/ntriples/"),
            Predicate::AbsoluteUri("http://purl.org/dc/elements/1.1/creator"),
            Object::Literal{value:"Dave Beckett",language:"",datatype:""}
        ))
    );
    assert_eq!(
        triple(&b"<http://www.w3.org/2001/sw/RDFCore/ntriples/>    <http://purl.org/dc/elements/1.1/creator> \"Dave Beckett\" ."[..]),
        IResult::Done(&b""[..], MyTriple::new(
            Subject::AbsoluteUri("http://www.w3.org/2001/sw/RDFCore/ntriples/"),
            Predicate::AbsoluteUri("http://purl.org/dc/elements/1.1/creator"),
            Object::Literal{value:"Dave Beckett",language:"",datatype:""}
        ))
    );
    assert_eq!(
        triple(&b"#Das ist ein Kommentar\n <http://www.w3.org/2001/sw/RDFCore/ntriples/>    <http://purl.org/dc/elements/1.1/creator> \"Dave Beckett\" ."[..]),
        IResult::Done(&b""[..], MyTriple::new(
            Subject::AbsoluteUri("http://www.w3.org/2001/sw/RDFCore/ntriples/"),
            Predicate::AbsoluteUri("http://purl.org/dc/elements/1.1/creator"),
            Object::Literal{value:"Dave Beckett",language:"",datatype:""}
        ))
    );
    assert_eq!(
        triple(&b"#Das ist ein Kommentar\n <http://www.w3.org/2001/sw/RDFCore/ntriples/>    <http://purl.org/dc/elements/1.1/creator> \"That Seventies Show\"^^<http://www.w3.org/2001/XMLSchema#string> ."[..]),
        IResult::Done(&b""[..], MyTriple::new(
            Subject::AbsoluteUri("http://www.w3.org/2001/sw/RDFCore/ntriples/"),
            Predicate::AbsoluteUri("http://purl.org/dc/elements/1.1/creator"),
            Object::Literal{value:"That Seventies Show",language:"",datatype:"http://www.w3.org/2001/XMLSchema#string"}
        ))
    );
    assert_eq!(
        triple(&b"#Das ist ein Kommentar\n <http://www.w3.org/2001/sw/RDFCore/ntriples/>    <http://purl.org/dc/elements/1.1/creator> \"That Seventies Show\"@en ."[..]),
        IResult::Done(&b""[..], MyTriple::new(
            Subject::AbsoluteUri("http://www.w3.org/2001/sw/RDFCore/ntriples/"),
            Predicate::AbsoluteUri("http://purl.org/dc/elements/1.1/creator"),
            Object::Literal{value:"That Seventies Show",language:"en",datatype:""}
        ))
    );
}
