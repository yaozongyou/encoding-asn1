/*
World-Schema DEFINITIONS IMPLICIT TAGS ::=
BEGIN
Request ::= SEQUENCE {
  num INTEGER
}
Response ::= SEQUENCE {
  ret INTEGER
}
BODY ::= CHOICE {
  request [3000] EXPLICIT  Request,
  response [3001] EXPLICIT  Response
}
Message ::= SEQUENCE {
  id INTEGER,
  body BODY
}
END

value Message ::= {
  id 1,
  body request : {
    num 1
  }
}
*/
use encoding_asn1::{common, Encoder, Marshal, Marshaler};

#[derive(Debug, Marshal)]
struct Message {
    seq: i32,
    body: Body,
}

#[derive(Debug, Marshal)]
#[allow(dead_code)]
enum Body {
    #[asn1(tag = 3000)]
    Request(Request),

    #[asn1(tag = 3001)]
    Response(Response),
}

#[derive(Debug, Marshal)]
struct Request {
    num: i32,
}

#[derive(Debug, Marshal)]
struct Response {
    ret: i32,
}

#[test]
fn it_works() {
    assert_eq!(
        encoding_asn1::marshal(&Message {
            seq: 1,
            body: Body::Request(Request { num: 1 })
        }),
        vec![0x30, 0x0C, 0x02, 0x01, 0x01, 0xBF, 0x97, 0x38, 0x05, 0x30, 0x03, 0x02, 0x01, 0x01]
    );
}
