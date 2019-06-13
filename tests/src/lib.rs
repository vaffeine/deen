use deen::{Any, Tag, U16be, U8};
use deen_proc::deen;

deen! {
    #[derive(Debug, Clone, PartialEq)]
    struct TpktHeader <- TpktEncoder(version: u8) {
        Tag::new(U8, version),
        Any::new(U8),
        length <- U16be,
        Tag::new(U16be, u16::from(version) + length),
    }
}

#[cfg(test)]
impl TpktHeader {
    fn new(length: u16) -> Self {
        Self { length }
    }
}

#[test]
fn encode_tpkt() {
    let mut buf = Vec::new();
    TpktEncoder { version: 3 }
        .encode(&TpktHeader::new(0x542), &mut buf)
        .unwrap();
    assert_eq!(&buf, &[0x3, 0x0, 0x5, 0x42, 0x5, 0x45]);
}

#[test]
fn decode_valid_tpkt() {
    let buf = vec![0x3, 0x0, 0x5, 0x42, 0x5, 0x45];
    let t = TpktEncoder { version: 3 }
        .decode(&mut buf.as_slice())
        .unwrap();
    assert_eq!(t, TpktHeader::new(0x542));
}

#[test]
fn decode_invalid_tpkt() {
    let buf = vec![0x8, 0x0, 0x5, 0x42, 0x5, 0x45];
    let res = TpktEncoder { version: 3 }.decode(&mut buf.as_slice());
    assert_eq!(res.unwrap_err().kind(), io::ErrorKind::InvalidData);
}
