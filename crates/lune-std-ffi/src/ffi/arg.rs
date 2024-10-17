use super::FfiConvert;

pub struct FfiArgRefOption {
    pub flag: u8,
}

pub enum FfiArgType {
    FfiBox,
    FfiRef(FfiArgRefOption),
}

pub struct FfiArgInfo {
    pub conv: *const dyn FfiConvert,
    pub size: usize,
    // pub kind: NativeArgType,
}
