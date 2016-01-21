#[macro_use]
extern crate nom;

use nom::{IResult, alpha, alphanumeric};


fn is_absolute_uri_char(c: u8) -> bool{
    c != b'>'
}

named!(absolute_uri, take_while!(is_absolute_uri_char));

named!(pub uriref <&[u8]>, chain!(
        char!('<') ~
        uri: absolute_uri ~
        char!('>'),
        ||{
            uri
        }
    )
);

fn is_no_newline(b: u8) -> bool{
    match b{
        b'\n' => false,
        _ => true
    }
}

named!(pub comment <&[u8]>,
    preceded!(
        tag!("#"),
        take_while!(is_no_newline)
    )
);

named!(name, call!(alphanumeric));

named!(pub named_node <&[u8]>,
    preceded!(
        tag!("_:"),
        name
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
        absolute_uri(b"http://test"),
        IResult::Done(&b""[..],&b"http://test"[..])
    );
    assert_eq!(
        uriref(b"<http://test>"),
        IResult::Done(&b""[..],&b"http://test"[..])
    );
    assert_eq!(
        comment(b"#test wie das geht \n"),
        IResult::Done(&b"\n"[..],&b"test wie das geht "[..])
    );
    assert_eq!(
        name(b"Der92Name"),
        IResult::Done(&b""[..],&b"Der92Name"[..])
    );
    assert_eq!(
        named_node(b"_:name4Node"),
        IResult::Done(&b""[..],&b"name4Node"[..])
    );
}
