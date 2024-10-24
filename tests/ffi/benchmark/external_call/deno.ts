import { libSuffix } from "../../utils/libSuffix.ts";
import { get_clock, get_offset } from "../../utils/proc_clock/deno.ts";

const library_file = "./tests/ffi/benchmark/external_call/lib."+libSuffix;
// @ts-ignore
let library = Deno.dlopen(library_file, {
    add: {
        parameters: ["i32", "i32"],
        result: "i32",
    },
});

function bench_add(bench_size: number) {
    let add = library.symbols.add;
    let value = 0;
    const before = get_clock();
    for (let i=0; i<bench_size; i++) {
        value = add(value,1);
    }
    const after = get_clock();
    console.log(get_offset(before,after))
}

bench_add(1000000);