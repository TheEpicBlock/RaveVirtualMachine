use crate::byte_util::{ByteParseable, BigEndianReadExt};
use crate::ClassParseError;
use std::io::{Cursor, Read, Seek, SeekFrom, Take};
use std::ops::{Range, RangeFrom, RangeFull, RangeTo};
use std::slice::SliceIndex;

macro_rules! ignore {
    ($a:ident) => {};
}

macro_rules! gen_bytecode_enum {
    (
        $(#[$Meta:meta])*
        pub enum $Name:ident {
            $(
                $($(#[$InstrMeta:meta])* $Instr:ident$(($($innerType:ident),*))? = $InstrHex:literal)?
                $(|$Result:ident$(($($Value:literal),*))? = $PHInstrHex:literal)?
                $(&$NameInternal:ident$(($($ValueInternal:ident),*))?)?
            ,)*
        }
    ) => {
        $(#[$Meta])*
        pub enum $Name {
            $(
                $($(#[$InstrMeta])* $Instr$(($($innerType),+))?,)?
            )*
            $(
                $($NameInternal$(($($ValueInternal),+))?,)?
            )*
        }

        impl ByteParseable for $Name {
            fn parse(bytes: &mut impl Read) -> Result<Self, ClassParseError> {
                let code = bytes.read_u8()?;
                match code {
                    $(
                        $($InstrHex => Ok($Name::$Instr$(($($innerType::parse(bytes)?),*))?),)?
                        $($PHInstrHex => Ok($Name::$Result$(($($Value),*))?),)?
                    )*
                    _ => Err(ClassParseError::InvalidBytecode(code))
                }
            }
        }

        impl $Name {
            /// Length of this instruction in bytes
            pub fn byte_size(&self) -> usize {
                match self {
                    $(
                        // SAFETY: core::mem::size_of needs to return the same amount as what ByteParseable parses
                        // TODO: might be better to add a `size` field to ByteParseable instead of querying core::mem::size_of
                        $($Name::$Instr$((..) if { $(ignore!($innerType);)* true })? => [1,$($(core::mem::size_of::<$innerType>()),*)?].iter().sum(),)?
                        $($Name::$NameInternal(..) => [1,$($(core::mem::size_of::<$ValueInternal>()),*)?].iter().sum(),)?
                    )*
                    _ => unreachable!()
                }
            }
        }
    }
}

gen_bytecode_enum! {
    /// https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-6.html#jvms-6.5
    #[derive(Debug, Clone)]
    #[allow(non_camel_case_types)]
    pub enum Instruction {
        /// Load onto the stack a reference from an array
        AALoad = 0x32,
        /// Store a reference in an array
        AAStore = 0x53,
        AConstNull = 0x01,
        ALoad(u8) = 0x19,
        |ALoad(0) = 0x2a,
        |ALoad(1) = 0x2b,
        |ALoad(2) = 0x2c,
        |ALoad(3) = 0x2d,
        ANewArray(u16) = 0xbd,
        AReturn = 0xb0,
        ArrayLength = 0xbe,
        AStore(u8) = 0x3a,
        |AStore(0) = 0x4b,
        |AStore(1) = 0x4c,
        |AStore(2) = 0x4d,
        |AStore(3) = 0x4e,
        AThrow = 0xbf,
        BALoad = 0x33,
        BAStore = 0x54,
        BIPush(u8) = 0x10,
        Breakpoint = 0xca,
        CALoad = 0x34,
        CAStore = 0x55,
        Checkcast(u16) = 0xc0,
        D2F = 0x90,
        D2I = 0x8e,
        D2L = 0x8f,
        DAdd = 0x63,
        DALoad = 0x31,
        DAStore = 0x52,
        DCmpG = 0x98,
        DCmpL = 0x97,
        &DConst(f64),
        |DConst(0.0) = 0x0e,
        |DConst(1.0) = 0x0f,
        DDiv = 0x6f,
        DLoad(u8) = 0x18,
        |DLoad(0) = 0x26,
        |DLoad(1) = 0x27,
        |DLoad(2) = 0x28,
        |DLoad(3) = 0x29,
        DMul = 0x6b,
        DNeg = 0x77,
        DRem = 0x73,
        DReturn = 0xaf,
        DStore(u8) = 0x39,
        |DStore(0) = 0x47,
        |DStore(1) = 0x48,
        |DStore(2) = 0x49,
        |DStore(3) = 0x4a,
        DSub = 0x67,
        Dup = 0x59,
        Dup_x1 = 0x5a,
        Dup_x2 = 0x5b,
        Dup2 = 0x5c,
        Dup2_x1 = 0x5d,
        Dup2_x2 = 0x5e,
        F2D = 0x8d,
        FSI = 0x8b,
        F2L = 0x8c,
        FAdd = 0x62,
        FAload = 0x30,
        FAstore = 0x51,
        FCmpG = 0x96,
        FCmpL = 0x95,
        &FConst(f32),
        |FConst(0.0) = 0x0b,
        |FConst(1.0) = 0x0c,
        |FConst(2.0) = 0x0d,
        FDiv = 0x6e,
        FLoad(u8) = 0x17,
        |FLoad(0) = 0x22,
        |FLoad(1) = 0x23,
        |FLoad(2) = 0x24,
        |FLoad(3) = 0x25,
        FMul = 0x6a,
        FNeg = 0x76,
        FRem = 0x72,
        FReturn = 0xae,
        FStore(u8) = 0x38,
        |FStore(0) = 0x43,
        |FStore(1) = 0x44,
        |FStore(2) = 0x45,
        |FStore(3) = 0x46,
        FSub = 0x66,
        GetField(u16) = 0xb4,
        GetStatic(u16) = 0xb2,
        Goto(i16) = 0xa7,
        Goto_w(u32) = 0xc8,
        I2B = 0x91,
        I2C = 0x92,
        I2D = 0x87,
        I2F = 0x86,
        I2L = 0x85,
        I2S = 0x93,
        IAdd = 0x60,
        IALoad = 0x2e,
        IAnd = 0x7e,
        IAstore = 0x4f,
        &IConst(i32),
        |IConst(-1) = 0x02,
        |IConst(0) = 0x03,
        |IConst(1) = 0x04,
        |IConst(2) = 0x05,
        |IConst(3) = 0x06,
        |IConst(4) = 0x07,
        |IConst(5) = 0x08,
        IDiv = 0x6c,
        IfACmpEq(u16) = 0xa5,
        IfACmpNe(u16) = 0xa6,
        IfICmpEq(u16) = 0x9f,
        IfICmpGe(u16) = 0xa2,
        IfICmpGt(u16) = 0xa3,
        IfICmpLe(u16) = 0xa4,
        IfICmpLt(u16) = 0xa1,
        IfICmpNe(u16) = 0xa0,
        IfEq(u16) = 0x99,
        IfGe(u16) = 0x9c,
        IfGt(u16) = 0x9d,
        IfLe(u16) = 0x9e,
        IfLt(u16) = 0x9b,
        IfNe(u16) = 0x9a,
        IfNonNull(u16) = 0xc7,
        IfNull(u16) = 0xc6,
        IInc(u8, i8) = 0x84,
        ILoad(u8) = 0x15,
        |ILoad(0) = 0x1a,
        |ILoad(1) = 0x1b,
        |ILoad(2) = 0x1c,
        |ILoad(3) = 0x1d,
        //ImpDep1 = 0xfe,
        //ImpDep2 = 0xff,
        IMul = 0x68,
        INeg = 0x74,
        InstanceOf(u16) = 0xc1,
        InvokeDynamic(u16, u16) = 0xba,
        InvokeInterface(u16, u16) = 0xb9,
        InvokeSpecial(u16) = 0xb7,
        InvokeStatic(u16) = 0xb8,
        InvokeVirtual(u16) = 0xb6,
        IOr = 0x80,
        IRem = 0x70,
        IReturn = 0xac,
        IShL = 0x78,
        IShR = 0x7a,
        IStore(u8) = 0x36,
        |IStore(0) = 0x3b,
        |IStore(1) = 0x3c,
        |IStore(2) = 0x3d,
        |IStore(3) = 0x3e,
        ISub = 0x64,
        IUShR = 0x7c,
        IXor = 0x82,
        JSr(u16) = 0xa8,
        JSr_w(u32) = 0xc9,
        L2D = 0x8a,
        L2F = 0x89,
        L2I = 0x88,
        LAdd = 0x61,
        LALoad = 0x2f,
        LanD = 0x7f,
        LAStore = 0x50,
        LCmp = 0x94,
        &LConst(i64),
        |LConst(0) = 0x09,
        |LConst(1) = 0x0a,
        LdC(u8) = 0x12,
        LdC_w(u16) = 0x13,
        LdC2_w(u16) = 0x14,
        LDiv = 0x6d,
        LLoad(u8) = 0x16,
        |LLoad(0) = 0x1e,
        |LLoad(1) = 0x1f,
        |LLoad(2) = 0x20,
        |LLoad(3) = 0x21,
        LMul = 0x69,
        LNeg = 0x75,
        //TODO LookupSwitch = 0xab,
        LOr = 0x81,
        LRem = 0x71,
        LReturn = 0xad,
        LShL = 0x79,
        LShR = 0x7b,
        LStore(u8) = 0x37,
        |LStore(0) = 0x3f,
        |LStore(1) = 0x40,
        |LStore(2) = 0x41,
        |LStore(3) = 0x42,
        LSub = 0x65,
        LUShR = 0x7d,
        LXor = 0x83,
        MonitorEnter = 0xc2,
        MonitorExit = 0xc3,
        MultiANewArray(u16, u8) = 0xc5,
        New(u16) = 0xbb,
        NewArray(u8) = 0xbc,
        Nop = 0x00,
        Pop = 0x57,
        Pop2 = 0x58,
        Putfield(u16) = 0xb5,
        PutStatic(u16) = 0xb3,
        /// Indexes the local variable table and jumps to what is contained in it
        Ret(u8) = 0xa9,
        Return = 0xb1,
        SALoad = 0x35,
        SAStore = 0x56,
        SIPush(u16) = 0x11,
        Swap = 0x5f,
        //TODO TableSwitch = 0xaa,
        //TODO Wide = 0xc4,
    }
}

#[cfg(test)]
mod tests {
    use crate::bytecode::Instruction;
    use crate::byte_util::ByteParseable;
    use crate::ClassParseError;
    use assert_matches::assert_matches;

    #[test]
    fn parse_iconst0() {
        let bytes = vec![0x03];
        let instr = Instruction::parse_bytes(&bytes).unwrap();
        assert_matches!(instr, Instruction::IConst(0));
    }

    #[test]
    fn parse_aload0() {
        let bytes = vec![0x2a];
        let instr = Instruction::parse_bytes(&bytes).unwrap();
        assert_matches!(instr, Instruction::ALoad(0));
    }

    #[test]
    fn parse_invokestatic() {
        let bytes = vec![0xb8, 0x12, 0x34];
        let result = Instruction::parse_bytes(&bytes).unwrap();

        assert_matches!(result, Instruction::InvokeStatic(0x1234));
    }

    #[test]
    fn parse_invalid() {
        let bytes = vec![0xfd];
        let result = Instruction::parse_bytes(&bytes);

        assert_matches!(result, Err(ClassParseError::InvalidBytecode(0xfd)));
    }
}

#[derive(Debug, Clone)]
pub struct Code {
    pub(crate) inner: Box<[u8]>,
}

pub trait IndexingRange<T> {
    fn get_or(self, v: Range<T>) -> Range<T>;
}

impl<T> IndexingRange<T> for RangeFull {
    fn get_or(self, v: Range<T>) -> Range<T> {
        v
    }
}

impl<T> IndexingRange<T> for Range<T> {
    fn get_or(self, v: Range<T>) -> Range<T> {
        self
    }
}

impl<T> IndexingRange<T> for RangeFrom<T> {
    fn get_or(self, v: Range<T>) -> Range<T> {
        self.start..v.end
    }
}

impl<T> IndexingRange<T> for RangeTo<T> {
    fn get_or(self, v: Range<T>) -> Range<T> {
        v.start..self.end
    }
}


impl Code {
    pub fn from_vec(v: Vec<u8>) -> Self {
        Self {
            inner: v.into()
        }
    }

    pub fn iter<'code, I: IndexingRange<usize>>(&'code self, range: I) -> CodeIterator<'code> {
        let range = range.get_or(0..self.inner.len());

        let cursor = Cursor::new(&self.inner[range.clone()]);

        CodeIterator { start: range.start, data: cursor }
    }

    pub fn byte_len(&self) -> usize {
        self.inner.len()
    }
}

pub struct CodeIterator<'code> {
    start: usize,
    data: Cursor<&'code [u8]>,
}

impl<'code> Iterator for CodeIterator<'code> {
    type Item = (usize, Instruction);

    fn next(&mut self) -> Option<Self::Item> {
        if self.data.is_empty() {
            None
        } else {
            Some((self.data.position() as usize + self.start, ByteParseable::parse(&mut self.data).unwrap()))
        }
    }
}