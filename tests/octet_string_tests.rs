/*
--<ASN1.HugeInteger World-Schema.Rocket.range>--
World-Schema DEFINITIONS AUTOMATIC TAGS ::=
BEGIN
  S ::= OCTET STRING
END

s S ::= '68656C6C6F'H
*/
use encoding_asn1::types;

#[test]
fn it_works() {
    let s: types::OctetString = "hello".as_bytes().to_vec();
    println!("s: {:?}", s);
    assert_eq!(
        encoding_asn1::marshal(&s),
        vec![0x04, 0x05, 0x68, 0x65, 0x6c, 0x6c, 0x6f]
    );
}
