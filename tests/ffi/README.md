# tests/ffi

## Requirements

gcc for library compiling (for external-\*)

## Test Results

**External tests**

- [x] [external_math](./external_math/init.luau)
- [x] [external_pointer](./external_pointer/init.luau)
- [x] [external_print](./external_print/init.luau)
- [x] [external_struct](./external_struct/init.luau)
- [x] [external_closure](./external_closure/init.luau)

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

## Benchmark Results

> Note: LuaJit's os.clock function returns process CPU time (used) which much smaller then Luau's os.clock output. In this benchmark, luau uses 'time.h' instead of os.clock. See [utility/proc_clock](./utility/proc_clock/init.luau)

### [benchmark/external_call](./benchmark/external_call/init.luau)

**Target external c function**

```c
int add(int a, int b) {
    return a + b;
}
```

bench_scale = 1000000

**Lune ffi call function**

> cargo run run tests/ffi/benchmark/external_call
> cargo run --profile=release run tests/ffi/benchmark/external_call

Lune release target: 0.205127 (sec)
Lune dev target: 1.556489 (sec)

**LuaJit ffi call function**

> luajit tests/ffi/benchmark/external_call/luajit.lua

LuaJIT 2.1.1727870382: 0.001682 (sec)
flags = JIT ON SSE3 SSE4.1 BMI2 fold cse dce fwd dse narrow loop abc sink fuse

**Deno ffi call function**

> deno run --unstable-ffi --allow-ffi ./tests/ffi/benchmark/external_call/deno.ts

Deno 1.46.3: 0.006384 (sec)
v8 = 12.9.202.5-rusty

**Sysinformation**

> CPU: AMD Ryzen 5 7600 (12) @ 5.1
> MEM: 61898MiB 5600 MT/s
> KERNEL: 6.8.12-2-pve (Proxmox VE 8.2.7 x86_64)
