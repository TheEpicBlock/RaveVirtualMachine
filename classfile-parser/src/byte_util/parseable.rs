use std::io::{Cursor, Read};
use crate::byte_util::BigEndianReadExt;
use crate::ClassParseError;

pub trait ByteParseable {
    fn parse_bytes(bytes: &[u8]) -> Result<Self, ClassParseError> where Self: Sized {
        return Self::parse(&mut Cursor::new(bytes));
    }

    fn parse(bytes: &mut impl Read) -> Result<Self, ClassParseError> where Self: Sized;
}

macro_rules! gen_primitive_impl {
    (
        $($Type:ty => $Function:path),+
    ) => {
        $(
            impl ByteParseable for $Type {
                fn parse(bytes: &mut impl Read) -> Result<Self, ClassParseError> where Self: Sized {
                    return Ok($Function(bytes)?);
                }
            }
        )+
    }
}

gen_primitive_impl! {
    u8  => BigEndianReadExt::read_u8,
    u16 => BigEndianReadExt::read_u16,
    u32 => BigEndianReadExt::read_u32,
    u64 => BigEndianReadExt::read_u64,
    i8  => BigEndianReadExt::read_i8,
    i16 => BigEndianReadExt::read_i16,
    i32 => BigEndianReadExt::read_i32,
    i64 => BigEndianReadExt::read_i64,
    f32 => BigEndianReadExt::read_f32,
    f64 => BigEndianReadExt::read_f64
}

///Creates a basic [ByteParseable] implementation
#[macro_export]
macro_rules! gen_parseable {
    (
        $(
            $(#[$outer:meta])*
            $Vis:vis struct $Name:ident {
                $(
                    $TypeVis:vis $Val:ident: $Type:ty,
                )+
            }
        )+
    ) => {
        $(
            $(#[$outer])*
            $Vis struct $Name {
                $(
                    $TypeVis $Val: $Type,
                )+
            }

            impl ByteParseable for $Name {
                fn parse(bytes: &mut impl Read) -> Result<Self, $crate::ClassParseError> {
                    Ok(
                        Self {
                            $(
                                $Val: ByteParseable::parse(bytes)?,
                            )+
                        }
                    )
                }
            }
        )+
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;
    use crate::byte_util::parseable::ByteParseable;
    use crate::gen_parseable;

    gen_parseable! {
        pub struct MacroTest {
            inner: u8,
        }
    }

    #[test]
    fn auto_macro_test() {
        let bytes = vec![0x56];
        let parsed = MacroTest::parse_bytes(&bytes).unwrap();
        assert_eq!(parsed.inner, 0x56)
    }
}
