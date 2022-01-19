pub const TAG_BOOLEAN: i32 = 1;
pub const TAG_INTEGER: i32 = 2;
pub const TAG_BIT_STRING: i32 = 3;
pub const TAG_OCTET_STRING: i32 = 4;
pub const TAG_NULL: i32 = 5;
pub const TAG_OID: i32 = 6;
pub const TAG_ENUM: i32 = 10;
pub const TAG_UTF8_STRING: i32 = 12;
pub const TAG_SEQUENCE: i32 = 16;
pub const TAG_SET: i32 = 17;
pub const TAG_NUMERIC_STRING: i32 = 18;
pub const TAG_PRINTABLE_STRING: i32 = 19;
pub const TAG_T61_STRING: i32 = 20;
pub const TAG_IA5_STRING: i32 = 22;
pub const TAG_UTCTIME: i32 = 23;
pub const TAG_GENERALIZED_TIME: i32 = 24;
pub const TAG_GENERAL_STRING: i32 = 27;
pub const TAG_BMPSTRING: i32 = 30;

// ASN.1 class types represent the namespace of the tag.
pub const CLASS_UNIVERSAL: i32 = 0;
pub const CLASS_APPLICATION: i32 = 1;
pub const CLASS_CONTEXT_SPECIFIC: i32 = 2;
pub const CLASS_PRIVATE: i32 = 3;

// ASN.1 has IMPLICIT and EXPLICIT tags, which can be translated as "instead
// of" and "in addition to". When not specified, every primitive type has a
// default tag in the UNIVERSAL class.
//
// For example: a BIT STRING is tagged [UNIVERSAL 3] by default (although ASN.1
// doesn't actually have a UNIVERSAL keyword). However, by saying [IMPLICIT
// CONTEXT-SPECIFIC 42], that means that the tag is replaced by another.
//
// On the other hand, if it said [EXPLICIT CONTEXT-SPECIFIC 10], then an
// /additional/ tag would wrap the default tag. This explicit tag will have the
// compound flag set.
//
// (This is used in order to remove ambiguity with optional elements.)
//
// You can layer EXPLICIT and IMPLICIT tags to an arbitrary depth, however we
// don't support that here. We support a single layer of EXPLICIT or IMPLICIT
// tagging with tag strings on the fields of a structure.

// FieldParameters is the parsed representation of tag string from a structure field.
#[derive(Debug)]
pub struct FieldParameters {
    pub optional: bool,             // true iff the field is OPTIONAL
    pub explicit: bool,             // true iff an EXPLICIT tag is in use.
    pub application: bool,          // true iff an APPLICATION tag is in use.
    pub private: bool,              // true iff a PRIVATE tag is in use.
    pub default_value: Option<i64>, // a default value for INTEGER typed fields (maybe nil).
    pub tag: Option<i32>,           // the EXPLICIT or IMPLICIT tag (maybe nil).
    pub string_type: i32,           // the string tag to use when marshaling.
    pub time_type: i32,             // the time tag to use when marshaling.
    pub set: bool,                  // true iff this should be encoded as a SET
    pub omit_empty: bool,           // true iff this should be omitted if empty when marshaling.

                                    // Invariants:
                                    //   if explicit is set, tag is non-nil.
}

impl Default for FieldParameters {
    fn default() -> FieldParameters {
        FieldParameters {
            optional: false,
            explicit: false,
            application: false,
            private: false,
            default_value: None,
            tag: None,
            string_type: 0,
            time_type: 0,
            set: false,
            omit_empty: false,
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct TagAndLength {
    pub class: i32,
    pub tag: i32,
    pub length: usize,
    pub is_compound: bool,
}
