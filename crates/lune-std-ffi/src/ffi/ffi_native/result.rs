use super::NativeConvert;

// pub enum NativeResultType {
//     FfiBox,
//     FfiRef,
// }

pub struct NativeResultInfo {
    pub conv: *const dyn NativeConvert,
    pub size: usize,
    // kind: NativeResultType,
}
