use crate::common;
use crate::marshal;
use crate::marshal::Encoder;
use crate::unmarshal;

pub type OctetString = Vec<u8>;

#[derive(Debug)]
pub struct RawValue {
    pub class: i32,
    pub tag: i32,
    pub is_compound: bool,
    pub bytes: Vec<u8>,
    pub full_bytes: Vec<u8>, // includes the tag and length
}

impl marshal::Marshaler for RawValue {
    fn marshal_with_params(&self, _params: &common::FieldParameters) -> Vec<u8> {
        if self.full_bytes.len() > 0 {
            return self.full_bytes.to_vec();
        }

        let t = marshal::TaggedEncoder {
            tag: common::TagAndLength {
                class: self.class,
                is_compound: self.is_compound,
                length: self.bytes.len(),
                tag: self.tag,
            },
            body: self.bytes.to_vec(),
        };

        t.encode()
    }
}

impl unmarshal::Unmarshaler<RawValue> for RawValue {
    fn unmarshal_with_params<'a>(
        bytes: &'a [u8],
        _params: &common::FieldParameters,
    ) -> Result<(RawValue, &'a [u8]), unmarshal::Error> {
        println!("1111 bytes: {:02X?}", bytes);
        let (tag_and_length, bytes) = unmarshal::parse_tag_and_length(bytes)?;
        println!("tag_and_length: {:?}", tag_and_length);
        println!("bytes: {:02X?}", bytes);

        let rv = RawValue {
            class: tag_and_length.class,
            tag: tag_and_length.tag,
            is_compound: tag_and_length.is_compound,
            bytes: bytes[bytes.len() - tag_and_length.length..].to_vec(),
            full_bytes: bytes.to_vec(),
        };

        Ok((rv, &bytes[(tag_and_length.length as usize)..]))
    }
}
