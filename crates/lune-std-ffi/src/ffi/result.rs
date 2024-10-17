use super::FfiConvert;

// pub enum NativeResultType {
//     FfiBox,
//     FfiRef,
// }

pub struct FfiResultInfo {
    pub conv: *const dyn FfiConvert,
    pub size: usize,
    // kind: NativeResultType,
}
