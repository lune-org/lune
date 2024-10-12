use super::NativeConvert;

pub enum NativeResultType {
    FfiBox,
    FfiRef,
}

pub struct NativeResultInfo {
    conv: *const dyn NativeConvert,
    kind: NativeResultType,
}
