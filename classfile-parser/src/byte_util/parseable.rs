use std::io::{Cursor, Read};
use crate::byte_util::BigEndianReadExt;
use crate::ClassParseError;

pub trait ByteParseable {
    fn parse_bytes(bytes: &[u8]) -> Result<Self, ClassParseError> where Self: Sized {
        return Self::parse(&mut Cursor::new(bytes));
    }

    fn parse(bytes: &mut impl Read) -> Result<Self, ClassParseError> where Self: Sized;
}

pub fn parse_multiple<T: ByteParseable>(bytes: &mut impl Read, amount: usize) -> Result<Vec<T>, ClassParseError> {
    let mut result = Vec::with_capacity(amount);
    for _ in 0..amount {
        result.push(T::parse(bytes)?);
    }
    return Ok(result);
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
    f32 => BigEndianReadExt::read_f32,
    f64 => BigEndianReadExt::read_f64
}

///Creates a basic [ByteParseable] implementation
#[macro_export]
macro_rules! gen_parseable {
    (
        $(
            $(#[$outer:meta])*
            pub struct $Name:ident {
                $(
                    $Val:ident: $Type:ty,
                )+
            }
        )+
    ) => {
        $(
            $(#[$outer])*
            pub struct $Name {
                $(
                    $Val: $Type,
                )+
            }

            impl ByteParseable for $Name {
                fn parse(bytes: &mut impl Read) -> Result<Self, crate::ClassParseError> {
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
    use std::io::{Cursor, Read};

    use crate::byte_util::parseable::ByteParseable;
    use crate::byte_util::BigEndianReadExt;
    use crate::gen_parseable;

    #[derive(Eq, PartialEq, Debug)]
    struct Test(u8);

    impl ByteParseable for Test {
        fn parse(bytes: &mut impl Read) -> Result<Self, std::io::Error> {
            Ok(Test(bytes.read_u8()?))
        }
    }

    #[test]
    fn vector_byte_parse() {
        let bytes = vec![1, 2, 3, 5, 8];
        let mut tests = Vec::with_capacity(bytes.len());
        for i in &bytes {
            tests.push(Test(*i));
        }

        //Vector is now out original list of numbers. And byte_vector is the same but with the length appended at the front as a u16.
        assert_eq!(
            tests,
            Test::parse_array(&mut Cursor::new(bytes), tests.len()).unwrap()
        )
    }

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
