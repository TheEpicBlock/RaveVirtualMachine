use crate::byte_util::{ByteParseable, BigEndianReadExt};
use crate::class_file::attributing::attributes;
use crate::class_file::attributing::AttributingError;
use std::io::Read;

macro_rules! gen_bytecode_enum {
    (
        pub enum $Name:ident {
            $(
                $Instr:ident$(($($innerType:ident)+))? = $Value:expr,
            )+
        }
    ) => {
        pub enum $Name {
            $(
                $Instr$(($($innerType)+))?,
            )+
        }

        impl ByteParseable<attributes::Err> for $Name {
            fn parse(bytes: &mut impl Read) -> Result<Self, attributes::Err> {
                let code = bytes.read_u8()?;
                match code {
                    $(
                        $Value => Ok($Name::$Instr$(($($innerType::parse(bytes)?)+))?),
                    )+
                    _ => Err(AttributingError::InvalidBytecode(code))
                }
            }
        }
    }
}

gen_bytecode_enum!(
    pub enum Instruction {
        AALoad = 0x32,
        AAStore = 0x53,
        AConstNull = 0x01,
        ALoad = 0x19,
        ALoad0 = 0x2a,
        //TODO
        ANewArray = 0xbd,
        AReturn = 0xb0,
        ArrayLength = 0xbe,
        AStore = 0x3a,
        //TODO
        AThrow = 0xbf,
        //TODO
        InvokeStatic(u16) = 0xb7,
    }
);

#[cfg(test)]
mod tests {
    use crate::class_file::bytecode::Instruction;
    use crate::byte_util::ByteParseable;
    use std::io::Cursor;
    use crate::class_file::attributing::AttributingError;

    #[test]
    fn parse_invokestatic() {
        let bytes = vec![0xb7, 0x12, 0x34];
        let instr = Instruction::parse_bytes(&bytes).unwrap();
        assert!(matches!(instr, Instruction::InvokeStatic(0x1234)))
    }

    #[test]
    fn parse_invalid() {
        let bytes = vec![0x00];
        let instr = Instruction::parse_bytes(&bytes);

        assert!(matches!(instr, Err(AttributingError::InvalidBytecode(0x00))));
    }
}