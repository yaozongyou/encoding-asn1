use encoding_asn1::unmarshal::Error;
use encoding_asn1::{
    common, parse_tag_and_length, unmarshal_with_params, Encoder, Marshal, Marshaler, RawValue,
    Unmarshal, Unmarshaler,
};

#[derive(Debug, Marshal, Unmarshal)]
struct Message {
    id: i32,
    body: Body,
}

#[derive(Debug, Marshal, Unmarshal)]
#[allow(dead_code)]
enum Body {
    #[asn1(tag = 3000)]
    Request(Request),

    #[asn1(tag = 3001)]
    Response(Response),
}

/*
impl Unmarshaler<Body> for Body {
    fn unmarshal_with_params<'a>(
        bytes: &'a [u8],
        params: &common::FieldParameters,
    ) -> Result<(Body, &'a [u8]), Error> {
        println!("bytes: {:02X?}", bytes);

        let (rv, bytes) = unmarshal_with_params::<RawValue>(bytes, params)?;
        println!("222 bytes: {:02X?}", bytes);

        println!("rv: {:?}", rv);

        match rv.tag {
            3000 => {
                println!("aaaa");
                let (r, _) = unmarshal_with_params::<Request>(
                    &rv.bytes,
                    &common::FieldParameters {
                        ..common::FieldParameters::default()
                    },
                )?;
                Ok((Body::Request(r), bytes))
            }
            _ => {
                panic!()
            }
        }
    }
}
*/

#[derive(Debug, Marshal, Unmarshal)]
struct Request {
    num: i32,
}

#[derive(Debug, Marshal, Unmarshal)]
struct Response {
    ret: i32,
}

fn main() {
    let m = Message {
        id: 10,
        body: Body::Request(Request { num: 20 }),
    };
    let bytes = encoding_asn1::marshal(&m);
    println!("{:02X?}", bytes);

    let n = encoding_asn1::unmarshal::<Message>(&bytes);
    println!("{:?}", n);
}
