# tests/ffi

## Requirements

gcc for library compiling (for external-\*)

## Results

**External tests**

- [x] [external_math](./external_math/init.luau)
- [x] [external_pointer](./external_pointer/init.luau)
- [x] [external_print](./external_print/init.luau)
- [x] [external_struct](./external_struct/init.luau)
- [ ] [external_closure](./external_closure/init.luau)

  > failed (segfault)

**Luau-side**

- [ ] [pretty_print](./pretty_print)

  > need box, ref test

- [x] [isInteger](./isInteger)
- [ ] [into_boundary](./into_boundary)

  > need assertion

- [ ] [from_boundary](./from_boundary)

  > need assertion

- [ ] [cast](./cast)

  > need assertion
