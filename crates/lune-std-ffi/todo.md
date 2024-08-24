use libffi::raw::{ffi_cif, ffi_ptrarray_to_raw};

// pub fn ffi_get_struct_offsets(
// abi: ffi_abi,
// struct_type: *mut ffi_type,
// offsets: *mut usize,
// ) -> ffi_status;

- last thing to do
- [ ] Add tests
- [ ] Add docs
- [ ] Typing

# Raw

- [ ] Raw:toRef()
- [ ] Raw:toBox()
- [ ] Raw:intoBox()
- [ ] Raw:intoRef()

# Box

- [x] ffi.box(size)
- [x] .size
- [x] :zero()
- [?] :ref(offset?=0) => ref
  - offset is not impled
- [~] :copy(box,size?=-1,offset?=0)
  - working on it

# Ref (Unsafe)

- [ ] high, low Boundaries
- [ ] iter

- [x] ref:deref() -> ref
- [x] ref:offset(bytes) -> ref
- [x] ref:ref() -> ref

~~- [ ] ref:fromRef(size,offset?=0) ?? what is this~~
~~- [ ] ref:fromBox(size,offset?=0) ?? what is this~~

# Struct

- [x] :offset(index)
- [x] :ptr()
- [x] .inner[n]
- [!] .size
- [ ] #
- [x] tostring

size, offset is strange. maybe related to cif state.

# Type

- [ ] :toBox(luavalue)

Very stupid idea.
from(box|ref|raw, offset) is better idea i think.

- [ ] :fromBox(box,offset?=0)
- [ ] :intoBox(luavalue,box,offset?=0)
- [ ] :fromRef(ref,offset?=0)
- [ ] :intoRef(luavalue,ref,offset?=0)
- [ ] :fromRaw(raw,offset?=0)

- [ ] :castBox(box,type) TODO
- [ ]

- [ ] :sum
- [ ] :mul
- [ ] :sub

## subtype

- [x] :ptr() -> Ptr
- [~] :arr(len) -> Arr
- [x] .size

# Ptr

- [x] .inner
- [x] .size
- [x] :ptr()
- [~] :arr()

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
