<!-- markdownlint-disable MD036 -->
<!-- markdownlint-disable MD033 -->

# `tests/ffi`

## Requirements

gcc for library compiling (for external-\*)

## Test Results

**External tests**

- [x] [external_closure](./external_closure/init.luau)
- [x] [external_math](./external_math/init.luau)
- [x] [external_pointer](./external_pointer/init.luau)
- [x] [external_print](./external_print/init.luau)
- [x] [external_struct](./external_struct/init.luau)
- [x] [external_closure/errors](./external_closure/errors.luau) (an erroring closure does not crash the host)
- [x] [external_types/returns](./external_types/returns.luau) (integer-width returns, multi-arg calls, out-parameters, array-by-value args)
- [x] [external_types/variadic](./external_types/variadic.luau) (variadic `...` function calls)

**Luau-side**

- [x] [cast](./cast.luau)
- [x] [free](./free.luau)
- [x] [isInteger](./isInteger.luau)
- [x] [read_boundary](./readBoundary.luau)
- [x] [write_boundary](./writeBoundary.luau)
- [x] [numbers](./numbers.luau) (every numeric ctype roundtrip, incl. 16-byte u128/i128)
- [x] [cTypes](./cTypes.luau) (compiler-defined C type sizes, signedness, roundtrip)
- [x] [casts](./casts.luau) (numeric conversions: widen, truncate, sign change, float/int)
- [x] [boxHandling](./boxHandling.luau) (allocation, zeroing, refs, freeing lifecycle)
- [x] [arrays](./arrays.luau) (layout, whole-array/element access, large and struct arrays)
- [x] [structs](./structs.luau) (padded layout, nesting, copyData, introspection)
- [x] [pointers](./pointers.luau) (null checks, readRef/writeRef, pointer boxes)
- [x] [readCString](./readCString.luau) (null-terminated string reads)
- [x] [safety](./safety.luau) (bounds/permission guards reject invalid access)

**Types**

- [x] [arr](./types/arr.luau)
- [x] [ptr](./types/ptr.luau)
- [x] [struct](./types/struct.luau)

**Pretty Print**

- [x] [arr](./pretty_print/arr.luau)
- [x] [box](./pretty_print/box.luau)
- [x] [ref](./pretty_print/ref.luau)
- [x] [lib](./pretty_print/lib.luau)
- [x] [fn](./pretty_print/fn.luau)
- [x] [ptr](./pretty_print/ptr.luau)
- [x] [struct](./pretty_print/struct.luau)
- [x] [type](./pretty_print/type.luau)

## Benchmark Results

> Note: LuaJit's os.clock function returns process CPU time (used) which much smaller then Luau's os.clock output. In this benchmark, luau uses 'time.h' instead of os.clock. See [utility/proc_clock](./utility/proc_clock/init.luau)

<details><summary><h3><a href="./benchmark/external_call/init.luau">benchmark/external_call</a></h3></summary>

**Target external c function**

```c
int add(int a, int b) {
    return a + b;
}
```

bench_scale = 1000000

**Lune ffi**

Command: `cargo run run tests/ffi/benchmark/external_call`
Command: `cargo run --profile=release run tests/ffi/benchmark/external_call`

- Device1-Linux-PVE  
  Lune release target: 0.205127 (sec)  
  Lune dev target: 1.556489 (sec)

  > Commit: ddf0c4c

- Device2-Windows-11
  Lune release target: 0.1875 (sec)  
  Lune dev target: ? SEGFUALT (sec)

  > Commit: ddf0c4c

**C**

- Device1-Linux-PVE: 0.001949 (sec)
  > gcc (GCC) 14.2.1 20240910

**LuaJit ffi**

Command: `luajit tests/ffi/benchmark/external_call/luajit.lua`

- Device1-Linux-PVE: 0.001682 (sec)
  > LuaJIT 2.1.1727870382  
  > (flags = JIT ON SSE3 SSE4.1 BMI2 fold cse dce fwd dse narrow loop abc sink fuse)

**Deno ffi**

Command: `deno run --unstable-ffi --allow-ffi ./tests/ffi/benchmark/external_call/deno.ts`

- Device1-Linux-PVE: 0.006384 (sec)
  > Deno 1.46.3 (v8 = 12.9.202.5-rusty)

**Sysinformation**

- Device1-Linux-PVE

  > CPU: AMD Ryzen 5 7600 (12) @ 5.1  
  > MEM: 61898MiB 5600 MT/s  
  > KERNEL: 6.8.12-2-pve (Proxmox VE 8.2.7 x86_64)

- Device2-Windows-11

  > CPU: AMD Ryzen 5 7600 (4) @ 3.800GHz  
  > MEM: 12250MiB 5600 MT/s  
  > KERNEL: 10.0.22631 (Windows 11 x86_64)  
  > HOST: QEMU Standard PC (Q35 + ICH9, 2009)

</details>
