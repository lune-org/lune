import { libSuffix } from "../libSuffix.ts";

const library_file = "./tests/ffi/utils/proc_clock/lib."+libSuffix;
// @ts-ignore
let library = Deno.dlopen(library_file, {
    sizeof_clock: {
        parameters: [],
        result: "i32",
    },
});
const sizeof_clock = library.symbols.sizeof_clock();
const type_clock_t = "u" + (sizeof_clock * 8);
library.close();
// @ts-ignore
library = Deno.dlopen(library_file, {
    get_clock: {
        parameters: [],
        result: type_clock_t,
    },
    get_offset: {
        parameters: [type_clock_t, type_clock_t],
        result: "f64",
    },
});

export const get_clock = library.symbols.get_clock;
export const get_offset = library.symbols.get_offset;
