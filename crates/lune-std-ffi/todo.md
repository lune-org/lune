
use libffi::raw::{ffi_cif, ffi_ptrarray_to_raw};

// pub fn ffi_get_struct_offsets(
//     abi: ffi_abi,
//     struct_type: *mut ffi_type,
//     offsets: *mut usize,
// ) -> ffi_status;


# Raw

- [ ] Raw:toRef()  
- [ ] Raw:toBox()  
- [ ] Raw:intoBox()  
- [ ] Raw:intoRef()  

# Box

- [x] ffi.box(size)
- [ ] :zero()
- [ ] :copy(box,size?=-1,offset?=0)
- [x] .size
- [?] :ref(offset?=0) => ref

# Ref (Unsafe)

- [ ] ref:offset(bytes)
- [ ] ref:fromRef(size,offset?=0)
- [ ] ref:fromBox(size,offset?=0)

# Struct

지금 사이즈가 이상함
오프셋도 이상함
아마도 초기화 되어지지 않은 상태의 cif 전의 오브젝트라서?

pthread
promise
high level binding for ffi

# Type

- [ ] :toBox(luavalue)
- [ ] :fromBox(box,offset?=0)
- [ ] :intoBox(luavalue,box,offset?=0)
- [ ] :fromRef(ref,offset?=0)
- [ ] :intoRef(luavalue,ref,offset?=0)
- [ ] :fromRaw(raw,offset?=0)

- [ ] :sum
- [ ] :mul
- [ ] :sub

## subtype
- [x] :ptr() -> Ptr
- [ ] :arr(len) -> Arr
- [x] .size

- [ ] :cast(box,type) TODO

# Ptr

- [x] .inner
- [x] .size
- [x] :ptr()
- [ ] :arr()

## Arr

## Void

`ffi.void`

Zero sized type.

## Fn
Prototype type of some function. converts lua function into native function pointer or native function pointer into lua function.

`ffi.fn({ type }, type) -> fn`

:toLua( ref ) -> luafunction
:toBox( luafunction ) -> ref

> TODO: rust, and another ABI support
