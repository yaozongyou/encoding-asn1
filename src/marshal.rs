use crate::common;
pub use encoding_asn1_derive::Marshal;

pub trait Encoder {
    fn len(&self) -> usize {
        self.encode().len()
    }
    fn encode(&self) -> Vec<u8>;
}

pub struct TaggedEncoder<E1: Encoder, E2: Encoder> {
    pub tag: E1,
    pub body: E2,
}

impl<E1, E2> Encoder for TaggedEncoder<E1, E2>
where
    E1: Encoder,
    E2: Encoder,
{
    fn encode(&self) -> Vec<u8> {
        let mut v = self.tag.encode();
        v.append(&mut self.body.encode());
        v
    }
}

fn base128_int_length(mut n: i64) -> i32 {
    if n == 0 {
        return 1;
    }

    let mut l = 0;
    while n > 0 {
        l += 1;
        n >>= 7;
    }

    return l;
}

fn encode_int_using_base128(n: i64) -> Vec<u8> {
    let mut v = vec![];

    let l = base128_int_length(n);

    for i in (0..l).rev() {
        let mut o = (n >> ((i * 7) as u32)) as u8;
        o &= 0x7f;
        if i != 0 {
            o |= 0x80;
        }

        v.push(o);
    }

    v
}

fn length_length(mut i: i32) -> i32 {
    let mut num_bytes = 1;
    while i > 255 {
        num_bytes += 1;
        i >>= 8;
    }
    num_bytes
}

fn encode_length(i: i32) -> Vec<u8> {
    let mut v = vec![];
    let mut n = length_length(i);

    while n > 0 {
        v.push((i >> ((n - 1) * 8)) as u8);
        n -= 1;
    }
    v
}

impl Encoder for common::TagAndLength {
    fn encode(&self) -> Vec<u8> {
        let mut v = vec![];

        let mut b = (self.class as u8) << 6;
        if self.is_compound {
            b |= 0x20;
        }
        if self.tag >= 31 {
            b |= 0x1f;
            v.push(b);
            v.append(&mut encode_int_using_base128(self.tag as i64));
        } else {
            b |= self.tag as u8;
            v.push(b);
        }

        if self.length >= 128 {
            let l = length_length(self.length as i32);
            v.push(0x80 | l as u8);
            v.append(&mut encode_length(self.length as i32));
        } else {
            v.push(self.length as u8);
        }

        v
    }
}

impl Encoder for i32 {
    fn len(&self) -> usize {
        let mut i = *self;
        let mut n = 1;

        while i > 127 {
            n += 1;
            i >>= 8;
        }

        while i < -128 {
            n += 1;
            i >>= 8;
        }

        return n;
    }

    fn encode(&self) -> Vec<u8> {
        let mut v = vec![];
        let n = self.len();
        let i = *self;

        for j in 0..n {
            v.push((i >> ((n - 1 - j) * 8)) as u8);
        }

        v
    }
}

impl Encoder for Vec<u8> {
    fn len(&self) -> usize {
        self.len()
    }

    fn encode(&self) -> Vec<u8> {
        self.to_vec()
    }
}

pub trait Marshaler {
    fn marshal(&self) -> Vec<u8> {
        self.marshal_with_params(&common::FieldParameters::default())
    }
    fn marshal_with_params(&self, params: &common::FieldParameters) -> Vec<u8>;
}

impl Marshaler for i32 {
    fn marshal_with_params(&self, params: &common::FieldParameters) -> Vec<u8> {
        let mut class = common::CLASS_UNIVERSAL;
        let mut tag = common::TAG_INTEGER;
        if let Some(v) = params.tag {
            if params.application {
                class = common::CLASS_APPLICATION;
            } else if params.private {
                class = common::CLASS_PRIVATE
            } else {
                class = common::CLASS_CONTEXT_SPECIFIC
            }

            if params.explicit {
                let mut t = TaggedEncoder {
                    tag: common::TagAndLength {
                        class: class,
                        is_compound: true,
                        length: 0,
                        tag: v,
                    },
                    body: TaggedEncoder {
                        tag: common::TagAndLength {
                            class: 0,
                            is_compound: false,
                            length: self.len(),
                            tag: common::TAG_INTEGER,
                        },
                        body: self.encode(),
                    },
                };

                t.tag.length = t.body.len();

                return t.encode();
            }

            // implicit tag.
            tag = v;
        }

        let t = TaggedEncoder {
            tag: common::TagAndLength {
                class: class,
                is_compound: false,
                length: self.len(),
                tag: tag,
            },
            body: self.encode(),
        };

        t.encode()
    }
}

impl Marshaler for Vec<u8> {
    fn marshal_with_params(&self, _params: &common::FieldParameters) -> Vec<u8> {
        let t = TaggedEncoder {
            tag: common::TagAndLength {
                class: 0,
                is_compound: false,
                length: self.len(),
                tag: common::TAG_OCTET_STRING,
            },
            body: self.encode(),
        };

        t.encode()
    }
}

pub fn marshal<M: Marshaler>(m: &M) -> Vec<u8> {
    m.marshal()
}

pub fn marshal_with_params<M: Marshaler>(m: &M, params: &common::FieldParameters) -> Vec<u8> {
    m.marshal_with_params(params)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Marshal)]
    struct IntStruct {
        a: i32,
    }

    #[derive(Marshal)]
    struct TwoIntStruct {
        a: i32,
        b: i32,
    }

    #[derive(Marshal)]
    struct NestedStruct {
        a: IntStruct,
    }

    #[derive(Marshal)]
    struct ImplicitTagTest {
        #[asn1(implicit, tag = 5)]
        a: i32,
    }

    #[derive(Marshal)]
    struct ExplicitTagTest {
        #[asn1(explicit, tag = 5)]
        a: i32,
    }

    #[test]
    fn it_works() {
        assert_eq!(marshal(&10), vec![0x02, 0x01, 0x0a]);
        assert_eq!(marshal(&127), vec![0x02, 0x01, 0x7f]);
        assert_eq!(marshal(&128), vec![0x02, 0x02, 0x00, 0x80]);
        assert_eq!(marshal(&-128), vec![0x02, 0x01, 0x80]);
        assert_eq!(marshal(&-129), vec![0x02, 0x02, 0xff, 0x7f]);
        assert_eq!(
            marshal(&IntStruct { a: 64 }),
            vec![0x30, 0x03, 0x02, 0x01, 0x40]
        );
        assert_eq!(
            marshal(&TwoIntStruct { a: 64, b: 65 }),
            vec![0x30, 0x06, 0x02, 0x01, 0x40, 0x02, 0x01, 0x41]
        );
        assert_eq!(
            marshal(&NestedStruct {
                a: IntStruct { a: 127 }
            }),
            vec![0x30, 0x05, 0x30, 0x03, 0x02, 0x01, 0x7f]
        );
        assert_eq!(marshal(&vec![1, 2, 3]), vec![0x04, 0x03, 0x01, 0x02, 0x03]);
        assert_eq!(
            marshal(&ImplicitTagTest { a: 64 }),
            vec![0x30, 0x03, 0x85, 0x01, 0x40]
        );
        assert_eq!(
            marshal(&ExplicitTagTest { a: 64 }),
            vec![0x30, 0x05, 0xa5, 0x03, 0x02, 0x01, 0x40]
        );
        assert_eq!(
            marshal(&crate::types::RawValue {
                tag: 1,
                class: 2,
                is_compound: false,
                bytes: vec![0x01, 0x02, 0x03],
                full_bytes: vec![],
            }),
            vec![0x81, 0x03, 0x01, 0x02, 0x03]
        );
    }
}
