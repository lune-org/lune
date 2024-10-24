# `lune-std-ffi`

## Tests & Benchmarks

See [tests/ffi](../../tests/ffi/README.md)

## TODO

- Rewrite error messages
- Deref
- CString
- Add buffer for owned data support
- Add math operation.

  > Provide related methods: `CTypeInfo:add(target, from1, from2, ...)` and `:sub` `:mul` `:div` `:mod` `:pow` `:max` `:min` `:gt` `:lt`  
  > Luau cannot handle f64, i64 or i128, so we should provide math operation for it

- Add bit operation

  > Luau only supports 32bit bit operations

- Add wchar and wstring support

  > For windows API

- Add varargs support
- Array argument in cfn
- Ref boundary fix

## Code structure

### /c

Define C-ABI type information and provide conversion and casting

- [**Struct ` CArrInfo`:**](./src/c/struct_info.rs) Represents C Array type
- [**Struct ` CPtrInfo`:**](./src/c/ptr_info.rs) Represents C Pointer type
- [**Struct ` CFnInfo`:**](./src/c/fn_info.rs) Represents C Function signature
  > provide CallableData and ClosureData creation
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

**Structs:** Provide call information trait

- **Struct `FfiArg`:** Used for argument boundary checking and callback argument ref flag
- **Struct `FfiResult`:** Used for result boundary checking

**Trait `FfiData`:** Provide common data handle, including methods below

- **Method `check_inner_boundary`:** check boundary with offset and size
- **Method `get_inner_pointer`:** returns raw pointer `*mut ()`
- **Method `is_writable`**
- **Method `is_readable`**
- **Method `copy_from`** copy data from another data

- **Trait `FfiConvert`:** Provide methods for read LuaValue from FfiData or write LuaValue into FfiData

- **Method `value_into_data`:** set data with lua value
- **Method `value_from_data`:** get lua value from data
- **Method `copy_data`:** copy sized data into another data
- **Method `stringify_data`:** stringify data with specific type

> Note: `GetFfiData` trait in `data/mod.rs` provides `AnyUserData.get_data_handle() -> FfiData` method

**Mods:** Provide common helper functions

- [**Mod `association.rs`:**](./src/ffi/association.rs) GC utility, used for inner, ret and arg type holding in subtype
- [**Mod `bit_mask.rs`:**](./src/ffi/bit_mask.rs) u8 bitfield helper
- [**Mod `cast.rs`:**](./src/ffi/cast.rs) library
  - **Function `num_cast<From, Into>(from: FfiData, from: FfiData)`:**
    Cast number type value inno another number type
- [**Mod `libffi_helper.rs`:**](./src/ffi/libffi_helper.rs)
  - **Const `FFI_STATUS_NAMES`:** Used for `ffi_status` stringify
  - **Function `get_ensured_size`:** Returns ensured `ffi_type` size
  - **Const `SIZE_OF_POINTER`:** Platform specific pointer size (Compile time known)
  - **Function `ffi_status_assert`:** Convert `ffi_status` to `LuaResult<()>`
