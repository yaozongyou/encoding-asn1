use encoding_asn1::unmarshal::Error;
use encoding_asn1::{
    common, parse_tag_and_length, unmarshal, unmarshal_with_params, Unmarshal, Unmarshaler,
};

#[derive(Debug, Unmarshal)]
struct IntStruct {
    a: i32,
}

fn main() {
    let bytes = vec![0x30, 0x03, 0x02, 0x01, 0x40];
    let b = unmarshal::<IntStruct>(&bytes);
    println!("b: {:?}", b);
}
