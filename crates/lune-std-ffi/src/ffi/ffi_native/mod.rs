mod call;
mod cast;
mod convert;
mod data;

pub trait NativeSize {
    fn get_size(&self) -> usize;
}

pub trait NativeSignedness {
    fn get_signedness(&self) -> bool {
        false
    }
}

pub use self::{
    call::NativeCall, cast::native_num_cast, convert::NativeConvert, data::GetNativeData,
    data::NativeData,
};
