use std::io::{Cursor, Read};
use std::string::FromUtf8Error;
use std::{error, fmt};

pub trait ByteParseable<ERR: error::Error> {
    fn parse_bytes(bytes: &[u8]) -> Result<Self, ERR> where Self: Sized {
        return Self::parse(&mut Cursor::new(bytes));
    }

    fn parse(bytes: &mut impl Read) -> Result<Self, ERR> where Self: Sized;

    fn parse_array(bytes: &mut impl Read, amount: usize) -> Result<Vec<Self>, ERR> where Self: Sized {
        let mut res = Vec::with_capacity(amount);
        for _ in 0..amount {
            res.push(Self::parse(bytes)?);
        }
        return Ok(res);
    }
}

#[macro_export]
macro_rules! parseable_inner_parse {
    ($Reader:expr, u8) => { crate::byte_util::BigEndianReadExt::read_u8($Reader) };
    ($Reader:expr, u16) => { crate::byte_util::BigEndianReadExt::read_u16($Reader) };
    ($Reader:expr, u32) => { crate::byte_util::BigEndianReadExt::read_u32($Reader) };
    ($Reader:expr, u64) => { crate::byte_util::BigEndianReadExt::read_u64($Reader) };
    ($Reader:expr, f32) => { crate::byte_util::BigEndianReadExt::read_f32($Reader) };
    ($Reader:expr, f64) => { crate::byte_util::BigEndianReadExt::read_f64($Reader) };
    ($Reader:expr, $Type:ident) => { $Type::parse($Reader) };
}

///Creates a basic [ByteParseable] implementation
#[macro_export]
macro_rules! gen_parseable {
    (
        const ERR = $Err:path;
        $(
            $(#[$outer:meta])*
            pub struct $Name:ident {
                $(
                    $Val:ident: $Type:ident,
                )+
            }
        )+
    ) => {
        use $crate::parseable_inner_parse;
        $(
            $(#[$outer])*
            pub struct $Name {
                $(
                    $Val: $Type,
                )+
            }

            impl ByteParseable<$Err> for $Name {
                fn parse(bytes: &mut impl Read) -> Result<Self, $Err> {
                    Ok(
                        Self {
                            $(
                                $Val: parseable_inner_parse!(bytes, $Type)?,
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

    impl ByteParseable<std::io::Error> for Test {
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
        const ERR = std::io::Error;

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
