use deen::{Any, Optional, Tag, U16be, U32be, U32le, U8};
use deen_proc::deen;
use try_from_primitive::TryFromPrimitive;

#[derive(Debug, PartialEq)]
pub struct Header {
    version: u8,
    length: u16,
    foo: Option<Foo>,
}

#[repr(u16)]
#[derive(Debug, PartialEq, Copy, Clone, TryFromPrimitive)]
enum Foo {
    Hello = 0xff00,
    World = 0x00ff,
}

deen! {
    #[derive(Debug)]
    pub struct Encoder(magic: u32) for Header {
        Tag::new(U32be, magic),
        version ~ U8,
        Any::new(U8),
        length ~ U16be,
        foo ~ if version > 2 {
            Tag::new(U8, 0xff);
            Optional::<Foo>::wrap(U32be).decode_when(|| length > 1)
        } else if version > 1{
            Tag::new(U8, 0x00);
            Optional::<Foo>::wrap(U32le).decode_when(|| length > 1)
        } else {
            Optional::<Foo>::wrap(U32le).decode_when(|| length > 0)
        }
    }
}

#[cfg(test)]
impl Header {
    fn new(version: u8, length: u16, foo: Option<Foo>) -> Self {
        Self {
            version,
            length,
            foo,
        }
    }
}

#[test]
fn encode() {
    let mut buf = Vec::new();
    Encoder { magic: 0xcafebabe }
        .encode(&Header::new(3, 0x542, Some(Foo::Hello)), &mut buf)
        .unwrap();
    assert_eq!(
        &buf,
        &[0xca, 0xfe, 0xba, 0xbe, 0x03, 0x00, 0x05, 0x42, 0xff, 0x00, 0x00, 0xff, 0x00]
    );
}

#[test]
fn decode_valid() {
    let buf = vec![
        0xca, 0xfe, 0xba, 0xbe, 0x03, 0x00, 0x05, 0x42, 0xff, 0x00, 0x00, 0xff, 0x00,
    ];
    let t = Encoder { magic: 0xcafebabe }
        .decode(&mut buf.as_slice())
        .unwrap();
    assert_eq!(t, Header::new(3, 0x542, Some(Foo::Hello)));
}

#[test]
fn decode_invalid() {
    let buf = vec![
        0xca, 0xfe, 0xba, 0xbe, 0x03, 0x00, 0x05, 0x42, 0x00, 0x00, 0x09, 0x45,
    ];
    let err = Encoder { magic: 0xcafebabe }
        .decode(&mut buf.as_slice())
        .unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::InvalidData);
}
