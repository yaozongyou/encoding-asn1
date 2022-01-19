use encoding_asn1::{common, Encoder, Marshal, Marshaler};

#[derive(Debug, Marshal)]
struct Message {
    id: i32,
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

fn main() {
    let m = Message {
        id: 1,
        body: Body::Request(Request { num: 1 }),
    };
    println!("{:02X?}", encoding_asn1::marshal(&m));
}
