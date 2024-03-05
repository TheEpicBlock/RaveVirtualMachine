////////////////////
// Type constants //
////////////////////

pub type JavaInt = i32;
pub type JavaChar = i8;
pub type JavaFloat = f32;
pub type JavaVoid = ();

////////////
// Traits //
////////////

pub unsafe trait JavaCompatibleArgumentType {
    const DESCRIPTOR_FRAGMENT: &'static str;
}

pub unsafe trait JavaCompatibleReturnType {
    const DESCRIPTOR_FRAGMENT: &'static str;
}

pub unsafe trait JavaCompatibleFunction {
    const DESCRIPTOR: &'static str;
}

///////////
// Types //
///////////

// All arguments are also return types
unsafe impl<T> JavaCompatibleReturnType for T where T: JavaCompatibleArgumentType {
    const DESCRIPTOR_FRAGMENT: &'static str = T::DESCRIPTOR_FRAGMENT;
}

unsafe impl JavaCompatibleArgumentType for JavaInt {
    const DESCRIPTOR_FRAGMENT: &'static str = "I";
}

unsafe impl JavaCompatibleArgumentType for JavaChar {
    const DESCRIPTOR_FRAGMENT: &'static str = "C";
}

unsafe impl JavaCompatibleArgumentType for JavaFloat {
    const DESCRIPTOR_FRAGMENT: &'static str = "F";
}

unsafe impl JavaCompatibleReturnType for JavaVoid {
    const DESCRIPTOR_FRAGMENT: &'static str = "V";
}

///////////////
// Functions //
///////////////

macro_rules! compatible_function_impl {
    ($ret:ty) => {
        unsafe impl JavaCompatibleFunction for extern "C" fn () -> $ret {
            const DESCRIPTOR: &'static str = const_format::concatcp!("()", <$ret as JavaCompatibleReturnType>::DESCRIPTOR_FRAGMENT);
        }
    };
    ($param_a:ty, $ret:ty) => {
        unsafe impl JavaCompatibleFunction for extern "C" fn ($param_a) -> $ret {
            const DESCRIPTOR: &'static str = const_format::concatcp!("(", <$param_a as JavaCompatibleArgumentType>::DESCRIPTOR_FRAGMENT, ")", <$ret as JavaCompatibleReturnType>::DESCRIPTOR_FRAGMENT);
        }
    };
}

compatible_function_impl!(JavaInt);
compatible_function_impl!(JavaChar);
compatible_function_impl!(JavaFloat);
compatible_function_impl!(JavaVoid);
compatible_function_impl!(JavaInt, JavaInt);
compatible_function_impl!(JavaChar, JavaInt);
compatible_function_impl!(JavaFloat, JavaInt);
compatible_function_impl!(JavaInt, JavaChar);
compatible_function_impl!(JavaChar, JavaChar);
compatible_function_impl!(JavaFloat, JavaChar);
compatible_function_impl!(JavaInt, JavaVoid);
compatible_function_impl!(JavaChar, JavaVoid);
compatible_function_impl!(JavaFloat, JavaVoid);
compatible_function_impl!(JavaInt, JavaFloat);
compatible_function_impl!(JavaChar, JavaFloat);
compatible_function_impl!(JavaFloat, JavaFloat);