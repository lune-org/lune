# tests/ffi

## Requirements

gcc for library compiling (for external-\*)

## Results

**External tests**

- [x] tests/ffi/external-math
- [x] tests/ffi/external-pointer
- [x] tests/ffi/external-print
- [x] tests/ffi/external-struct
- [ ] tests/ffi/external-closure

  > failed (segfault)

**Luau-side**

- [ ] tests/ffi/pretty-print :white_check_mark:

  > need box, ref test

- [x] tests/ffi/isInteger
- [ ] tests/ffi/into-boundary

  > need assertion

- [ ] tests/ffi/from-boundary

  > need assertion

- [ ] tests/ffi/cast

  > need assertion
