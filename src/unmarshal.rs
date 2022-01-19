use crate::common;
pub use encoding_asn1_derive::Unmarshal;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("structural error")]
    StructuralError(String),

    #[error("Syntax error")]
    SyntaxError(String),
}

// parseBase128Int parses a base-128 encoded int from the given offset in the
// given byte slice. It returns the value and the new offset.
fn parse_base128_int(bytes: &[u8], init_offset: usize) -> Result<(i32, usize), Error> {
    let mut offset = init_offset;
    let mut ret64: i64 = 0;
    //for shifted := 0; offset < len(bytes); shifted++ {
    let mut shifted = 0;
    while offset < bytes.len() {
        // 5 * 7 bits per byte == 35 bits of data
        // Thus the representation is either non-minimal or too large for an int32
        if shifted == 5 {
            return Err(Error::StructuralError(
                "base 128 integer too large".to_string(),
            ));
        }
        ret64 <<= 7;
        let b = bytes[offset];
        // integers should be minimally encoded, so the leading octet should
        // never be 0x80
        if shifted == 0 && b == 0x80 {
            return Err(Error::SyntaxError(
                "integer is not minimally encoded".to_string(),
            ));
        }
        ret64 |= (b & 0x7f) as i64;
        offset += 1;
        if b & 0x80 == 0 {
            let ret = ret64 as i32;
            // Ensure that the returned value fits in an int on all platforms
            if ret64 > i32::MAX.into() {
                return Err(Error::SyntaxError("base 128 integer too large".to_string()));
            }
            return Ok((ret, offset));
        }

        shifted += 1;
    }
    return Err(Error::SyntaxError("truncated base 128 integer".to_string()));
}

// parseTagAndLength parses an ASN.1 tag and length pair from the given offset
// into a byte slice. It returns the parsed data and the new offset. SET and
// SET OF (tag 17) are mapped to SEQUENCE and SEQUENCE OF (tag 16) since we
// don't distinguish between ordered and unordered objects in this code.
pub fn parse_tag_and_length(bytes: &[u8]) -> Result<(common::TagAndLength, &[u8]), Error> {
    let mut ret = common::TagAndLength::default();
    let mut offset = 0;

    let mut b = bytes[offset];
    offset += 1;
    ret.class = (b >> 6) as i32;
    ret.is_compound = (b & 0x20) == 0x20;
    ret.tag = (b & 0x1f) as i32;

    // If the bottom five bits are set, then the tag number is actually base 128
    // encoded afterwards
    if ret.tag == 0x1f {
        let tmp = parse_base128_int(bytes, offset)?;
        ret.tag = tmp.0;
        offset = tmp.1;
        // Tags should be encoded in minimal form.
        if ret.tag < 0x1f {
            return Err(Error::SyntaxError("non-minimal tag".to_string()));
        }
    }

    b = bytes[offset];
    offset += 1;
    if b & 0x80 == 0 {
        // The length is encoded in the bottom 7 bits.
        ret.length = (b & 0x7f) as usize;
    } else {
        // Bottom 7 bits give the number of length bytes to follow.
        let num_bytes = (b & 0x7f) as i32;
        if num_bytes == 0 {
            //err = SyntaxError{"indefinite length found (not DER)"}
            //return
        }
        ret.length = 0;
        //for i := 0; i < numBytes; i++ {
        for _i in 0..num_bytes {
            //if offset >= len(bytes) {
            //    err = SyntaxError{"truncated tag or length"}
            //    return
            //}
            b = bytes[offset];
            offset += 1;
            //if ret.length >= 1<<23 {
            // We can't shift ret.length up without
            // overflowing.
            //    err = StructuralError{"length too large"}
            //    return
            //}
            ret.length <<= 8;
            ret.length |= b as usize;
            //if ret.length == 0 {
            // DER requires that lengths be minimal.
            //    err = StructuralError{"superfluous leading zeros in length"}
            //    return
            //}
        }
        // Short lengths must be encoded in short form.
        if ret.length < 0x80 {
            //err = StructuralError{"non-minimal length"}
            //return
        }
    }

    Ok((ret, &bytes[offset..]))
}

pub trait Unmarshaler<T> {
    fn unmarshal(bytes: &[u8]) -> Result<(T, &[u8]), Error> {
        Self::unmarshal_with_params(bytes, &common::FieldParameters::default())
    }
    fn unmarshal_with_params<'a>(
        bytes: &'a [u8],
        params: &common::FieldParameters,
    ) -> Result<(T, &'a [u8]), Error>;
}

pub fn unmarshal<T: Unmarshaler<T>>(bytes: &[u8]) -> Result<(T, &[u8]), Error> {
    T::unmarshal(bytes)
}

pub fn unmarshal_with_params<'a, T: Unmarshaler<T>>(
    bytes: &'a [u8],
    params: &common::FieldParameters,
) -> Result<(T, &'a [u8]), Error> {
    T::unmarshal_with_params(bytes, params)
}

pub fn parse_int32(bytes: &[u8]) -> i32 {
    let mut ret = 0;
    for bytes_read in 0..bytes.len() {
        ret <<= 8;
        ret |= bytes[bytes_read] as i32;
    }

    // Shift up and down in order to sign extend the result.
    ret <<= 32 - (bytes.len() as u8) * 8;
    ret >>= 32 - (bytes.len() as u8) * 8;
    ret
}

impl Unmarshaler<i32> for i32 {
    fn unmarshal_with_params<'a>(
        bytes: &'a [u8],
        _params: &common::FieldParameters,
    ) -> Result<(i32, &'a [u8]), Error> {
        println!("bytes: {:02X?}", bytes);

        let (tag_and_length, bytes) = parse_tag_and_length(bytes)?;
        println!("tag_and_length: {:?}", tag_and_length);
        println!("bytes: {:02X?}", bytes);
        let ret = parse_int32(&bytes[..tag_and_length.length as usize]);
        Ok((ret, &bytes[(tag_and_length.length as usize)..]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Unmarshal)]
    struct IntStruct {
        a: i32,
    }

    #[test]
    fn it_works() {
        assert_eq!(parse_int32(&vec![0x00]), 0);
        assert_eq!(parse_int32(&vec![0x7f]), 127);
        assert_eq!(parse_int32(&vec![0x00, 0x80]), 128);
        assert_eq!(parse_int32(&vec![0x01, 0x00]), 256);
        assert_eq!(parse_int32(&vec![0x80]), -128);
        assert_eq!(parse_int32(&vec![0xff, 0x7f]), -129);
        assert_eq!(parse_int32(&vec![0xff]), -1);
        assert_eq!(parse_int32(&vec![0x80, 0x00, 0x00, 0x00]), -2147483648);

        struct TagAndLengthTest {
            bytes: Vec<u8>,
            out: common::TagAndLength,
        }

        let tag_and_length_data = vec![
            TagAndLengthTest {
                bytes: vec![0x80, 0x01],
                out: common::TagAndLength {
                    class: 2,
                    length: 1,
                    ..common::TagAndLength::default()
                },
            },
            TagAndLengthTest {
                bytes: vec![0xa0, 0x01],
                out: common::TagAndLength {
                    class: 2,
                    length: 1,
                    is_compound: true,
                    ..common::TagAndLength::default()
                },
            },
            TagAndLengthTest {
                bytes: vec![0x02, 0x00],
                out: common::TagAndLength {
                    class: 0,
                    tag: 2,
                    length: 0,
                    is_compound: false,
                },
            },
            TagAndLengthTest {
                bytes: vec![0xfe, 0x00],
                out: common::TagAndLength {
                    class: 3,
                    tag: 30,
                    length: 0,
                    is_compound: true,
                },
            },
            TagAndLengthTest {
                bytes: vec![0x1f, 0x1f, 0x00],
                out: common::TagAndLength {
                    class: 0,
                    tag: 31,
                    length: 0,
                    is_compound: false,
                },
            },
            TagAndLengthTest {
                bytes: vec![0x1f, 0x81, 0x00, 0x00],
                out: common::TagAndLength {
                    class: 0,
                    tag: 128,
                    length: 0,
                    is_compound: false,
                },
            },
            TagAndLengthTest {
                bytes: vec![0x1f, 0x81, 0x80, 0x01, 0x00],
                out: common::TagAndLength {
                    class: 0,
                    tag: 0x4001,
                    length: 0,
                    is_compound: false,
                },
            },
            TagAndLengthTest {
                bytes: vec![0x00, 0x81, 0x80],
                out: common::TagAndLength {
                    class: 0,
                    tag: 0,
                    length: 128,
                    is_compound: false,
                },
            },
            TagAndLengthTest {
                bytes: vec![0x00, 0x82, 0x01, 0x00],
                out: common::TagAndLength {
                    class: 0,
                    tag: 0,
                    length: 256,
                    is_compound: false,
                },
            },
            TagAndLengthTest {
                bytes: vec![0xa0, 0x84, 0x7f, 0xff, 0xff, 0xff],
                out: common::TagAndLength {
                    class: 2,
                    tag: 0,
                    length: 0x7fffffff,
                    is_compound: true,
                },
            },
            TagAndLengthTest {
                bytes: vec![0x1f, 0x87, 0xFF, 0xFF, 0xFF, 0x7F, 0x00],
                out: common::TagAndLength {
                    class: 0,
                    tag: i32::MAX,
                    length: 0,
                    is_compound: false,
                },
            },
        ];

        for test in &tag_and_length_data {
            let (tl, _) = parse_tag_and_length(&test.bytes).unwrap();
            assert_eq!(tl, test.out);
        }

        let bytes = vec![0x02, 0x01, 0x42];
        let i = i32::unmarshal(&bytes).unwrap();
        assert_eq!(i.0, 0x42);

        let bytes = vec![0x30, 0x03, 0x02, 0x01, 0x40];
        let is = IntStruct::unmarshal(&bytes);
        println!("is: {:?}", is);
    }
}
