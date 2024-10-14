use super::NativeConvert;

// pub enum NativeResultType {
//     FfiBox,
//     FfiRef,
// }

pub struct NativeResultInfo {
    pub conv: *const dyn NativeConvert,
    // kind: NativeResultType,
}
