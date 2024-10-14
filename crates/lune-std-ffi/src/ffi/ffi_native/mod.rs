mod arg;
mod cast;
mod convert;
mod data;
mod result;

pub trait NativeSize {
    fn get_size(&self) -> usize;
}

pub trait NativeSignedness {
    fn get_signedness(&self) -> bool {
        false
    }
}

pub use self::{
    arg::FfiArgRefOption,
    arg::NativeArgInfo,
    arg::NativeArgType,
    cast::native_num_cast,
    convert::NativeConvert,
    data::GetNativeData,
    data::NativeData,
    result::NativeResultInfo,
    // result::NativeResultType,
};
