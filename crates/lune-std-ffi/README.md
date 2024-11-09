<!-- markdownlint-disable MD033 -->

# `lune-std-ffi`

## Tests & Benchmarks

See [tests/ffi](../../tests/ffi/README.md)

## TODO

- [CString](./src/c/string_info.rs)
- Add buffer as owned data support
- Add math operation for numeric types
  > Provide related methods: `CTypeInfo:add(target, from1, from2, ...)` and `:sub` `:mul` `:div` `:mod` `:pow` `:max` `:min` `:gt` `:lt`  
  > Luau cannot handle f64, i64 or i128, so we should provide math operation for it
- Add bit operation for box/ref
  > Luau only supports 32bit bit operations
- Add wchar and wstring support
  > For windows API support
- Add varargs support
- Array argument in cfn
- [More box/ref methods](./src/data/helper.rs)
  - writeString
  - readString
  - writeBase64
  - readBase64

## Code structure

### /c

Define C-ABI type information and provide conversion and casting

**Structs:** C ABI type informations

- [**Struct `CArrInfo`:**](./src/c/arr_info.rs) Represents C Array type
- [**Struct `CPtrInfo`:**](./src/c/ptr_info.rs) Represents C Pointer type
- [**Struct `CFnInfo`:**](./src/c/fn_info.rs) Represents C Function signature
  > provide `CallableData` and `ClosureData` creator
- [**Struct `CStructInfo`:**](./src/c/struct_info.rs) Represents C Struct type
- [**Struct `CTypeInfo<T>`:**](./src/c/type_info.rs) Represents C type, extended in `/c/types`

<details><summary><a href="./src/c/helper.rs"><strong>Mod <code>helper.rs</code>: C ABI type helper</strong></a></summary>

- **Function `get_conv`, `get_conv_list`:**
  get `FfiConvert` from userdata (CStruct, CArr, CPtr, CTypes)
- **Function `get_middle_type`, `get_middle_type_list`:**
  get **`libffi::middle::Type`:** from userdata (CFn, CStruct, CArr, CPtr, CTypes)
- **Function `get_size`:**
  get size from userdata
- **Function `has_void`:**
  check table has void type
- **Function `stringify`:**
  stringify any type userdata
- **Function `get_name`:**
  get type name from ctype userdata, used for pretty-print
- **Function `is_ctype`:** check userdata is ctype
- **Mod `method_provider`:** provide common userdata method implements

</details>

#### /c/types

Export fixed-size source time known types and non-fixed compile time known types
mod.rs implememts type-casting for all CTypes

<details><summary><a href="./src/c/types/mod.rs"><strong>Mod <code>ctype_helper</code>:</strong></a> c type helper</summary>

- **Function `get_conv`:**
  get `FfiConvert` from ctype userdata, used for struct and array conversion
- **Function `get_middle_type`:**
  get **`libffi::middle::Type`:** from ctype userdata
- **Function `get_size`:**
  get size from ctype userdata
- **Function `get_name`:**
  get type name from ctype userdata, used for pretty-print
- **Function `is_ctype`:** check userdata is ctype

</details>

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

<ul><li><details><summary><strong>Trait <code>FfiConvert</code>:</strong> Provide methods for read LuaValue from FfiData or write LuaValue into FfiData</summary>

- **Method `value_into_data`:** set data with lua value
- **Method `value_from_data`:** get lua value from data
- **Method `copy_data`:** copy sized data into another data
- **Method `stringify_data`:** stringify data with specific type

</details></li></ul>

**Structs:** Provide call information

- **Struct `FfiArg`:** Used for argument boundary checking and callback argument ref flag
- **Struct `FfiResult`:** Used for result boundary checking

<details><summary><strong>Trait <code>FfiData</code>:</strong> Provide common data handle, including methods below</summary>

- **Method `check_inner_boundary`:** check boundary with offset and size
- **Method `get_inner_pointer`:** returns raw pointer `*mut ()`
- **Method `is_writable`**
- **Method `is_readable`**
- **Method `copy_from`** copy data from another data

</details>

> Note: `GetFfiData` trait in `data/mod.rs` provides `(LuaValue | LuaAnyUserData).get_data_handle() -> FfiData` method

**Mods:** Provide common helper functions

- [**Mod `association.rs`:**](./src/ffi/association.rs) GC utility, used for inner, ret and arg type holding in subtype
- [**Mod `bit_mask.rs`:**](./src/ffi/bit_mask.rs) u8 bitfield helper
- [**Mod `cast.rs`:**](./src/ffi/cast.rs) num cast library wrapper
  - **Function `num_cast<From, Into>(from: FfiData, from: FfiData)`:**
    Cast number type value inno another number type

<ul><li><details><summary><a href="./src/c/struct_info.rs"><strong>Mod <code>libffi_helper.rs</code>:</strong></a> libffi library helper</summary>

- **Const `FFI_STATUS_NAMES`:** Stringify `ffi_status`
- **Function `get_ensured_size`:** Returns ensured size of `ffi_type`
- **Const `SIZE_OF_POINTER`:** Platform specific pointer size (Compile time known)
- **Function `ffi_status_assert`:** Convert `ffi_status` to `LuaResult<()>`

</details></li></ul>
