use std::io;

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::Deen;

macro_rules! deen_integer {
    ($name:ident, $type:ty, $wr:ident, $rd:ident, $endian:ident) => {
        #[derive(Clone, Copy, Debug)]
        pub struct $name;

        impl Deen for $name {
            type Item = $type;

            fn encode(&self, value: &Self::Item, mut buf: impl io::Write) -> io::Result<()> {
                buf.$wr::<$endian>(*value)
            }
            fn decode(&self, mut buf: impl io::Read) -> io::Result<Self::Item> {
                buf.$rd::<$endian>()
            }
        }
    };
}

#[derive(Clone, Copy, Debug)]
pub struct U8;

impl Deen for U8 {
    type Item = u8;

    fn encode(&self, value: &Self::Item, mut buf: impl io::Write) -> io::Result<()> {
        buf.write_u8(*value)
    }
    fn decode(&self, mut buf: impl io::Read) -> io::Result<Self::Item> {
        buf.read_u8()
    }
}

deen_integer!(U16be, u16, write_u16, read_u16, BigEndian);
deen_integer!(U16le, u16, write_u16, read_u16, LittleEndian);
deen_integer!(U24be, u32, write_u24, read_u24, BigEndian);
deen_integer!(U24le, u32, write_u24, read_u24, LittleEndian);
deen_integer!(U32be, u32, write_u32, read_u32, BigEndian);
deen_integer!(U32le, u32, write_u32, read_u32, LittleEndian);
deen_integer!(U48be, u64, write_u48, read_u48, BigEndian);
deen_integer!(U48le, u64, write_u48, read_u48, LittleEndian);
deen_integer!(U64be, u64, write_u64, read_u64, BigEndian);
deen_integer!(U64le, u64, write_u64, read_u64, LittleEndian);
deen_integer!(U128be, u128, write_u128, read_u128, BigEndian);
deen_integer!(U128le, u128, write_u128, read_u128, LittleEndian);

deen_integer!(I16be, i16, write_i16, read_i16, BigEndian);
deen_integer!(I16le, i16, write_i16, read_i16, LittleEndian);
deen_integer!(I24be, i32, write_i24, read_i24, BigEndian);
deen_integer!(I24le, i32, write_i24, read_i24, LittleEndian);
deen_integer!(I32be, i32, write_i32, read_i32, BigEndian);
deen_integer!(I32le, i32, write_i32, read_i32, LittleEndian);
deen_integer!(I48be, i64, write_i48, read_i48, BigEndian);
deen_integer!(I48le, i64, write_i48, read_i48, LittleEndian);
deen_integer!(I64be, i64, write_i64, read_i64, BigEndian);
deen_integer!(I64le, i64, write_i64, read_i64, LittleEndian);
deen_integer!(I128be, i128, write_i128, read_i128, BigEndian);
deen_integer!(I128le, i128, write_i128, read_i128, LittleEndian);
