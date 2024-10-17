# `lune-std-ffi`

## Code structure

### /c

Define C-ABI type information and provide conversion and casting

- [**Struct ` CArrInfo`:**](./src/c/struct_info.rs) Represents C Array type
- [**Struct ` CPtrInfo`:**](./src/c/ptr_info.rs) Represents C Pointer type
- [**Struct ` CFnInfo`:**](./src/c/fn_info.rs) Represents C Function signature
  provide CallableData and ClosureData creation
- [**Struct ` CStructInfo`:**](./src/c/struct_info.rs) Represents C Struct type
- [**Struct ` CTypeInfo<T>`:**](./src/c/type_info.rs) Represents C type, extended in `/c/types`

#### /c/types

Export fixed-size source time known types and non-fixed compile time known types
Implememt type-casting for all CTypes

**Mod `ctype_helper`:**

- **Function `get_conv`:**
  get _FfiConvert_ from some ctype userdata, used for struct and array conversion
- **Function `get_size`:**
  get size from some ctype userdata, used for call return and arguments boundary checking
- **Function `get_name`:**
  get type name from some ctype userdata, used for pretty-print
- **Function `get_middle_type`:**
  get **`libffi::middle::Type`:** from some ctype userdata
- **Function `is_ctype`:** check userdata is ctype

---

### /data

**Structs:** Provide memory userdata

- [**Struct `BoxData`:**](./src/data/box_data/mod.rs) A heap allocated memory with user definable lifetime
- [**Struct `LibData`:**](./src/data/lib_data.rs) A dynamic opened library
- [**Struct `RefData`:**](./src/data/ref_data/mod.rs) A reference that can be used for receiving return data from external function or pass pointer arguments

**Structs:** Provide function(pointer) userdata

- [**Struct `CallableData`:**](./src/data/callable_data.rs) A callable function, which can be created from function pointer
- [**Struct `ClosureData`:**](./src/data/closure_data.rs) A closure pointer, which can be created from lua function and can be used for callback

---

### /ffi

**Traits:** Provide ABI shared common type information trait

- **Trait `FfiSize`**
- **Trait `FfiSignedness`**
- **Trait `FfiConvert`:** Provide read LuaValue from FfiData or write LuaValue into FfiData

**Trait `FfiData`:** Provide common data handle, including methods below

- **Method `check_boundary`:** check boundary with offset and size
- **Method `get_pointer`:** returns raw pointer `*mut ()`
- **Method `is_writable`**
- **Method `is_readable`**

> Note: `GetFfiData` trait in `data/mod.rs` provides `AnyUserData.get_data_handle() -> FfiData` method

**Mods:** Provide common helper functions

- **`association.rs`:** GC utility, used for inner, ret and arg type holding in subtype
- **`bit_mask.rs`:** u8 bitfield helper
- **`cast.rs`:** library
  - **Function `num_cast<From, Into>(from: FfiData, from: FfiData)`:**
    Cast number type value inno another number type
- **`libffi_helper.rs`:**
  - **Const `FFI_STATUS_NAMES`:** Used for ffi_status stringify
  - **Function `get_ensured_size`:** Returns ensured ffi_type size
  - **Const `SIEE_OF_POINTER`:** Platform specific pointer size (Compile time known)

## TODO
