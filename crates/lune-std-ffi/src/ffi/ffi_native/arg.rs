use super::NativeConvert;

pub struct FfiArgRefOption {
    pub flag: u8,
}

pub enum NativeArgType {
    FfiBox,
    FfiRef(FfiArgRefOption),
}

pub struct NativeArgInfo {
    pub conv: *const dyn NativeConvert,
    // pub kind: NativeArgType,
}
