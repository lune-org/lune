local CType = {}
-- NOTE: T is a unique identifier for the `CType` and R is the closest Lua type.
export type CType<T, R> = typeof(CType) & {
    size: number,
    signedness: boolean,
} & { ["__phantom"]: T }

function CType.ptr<T, R>(self: CType<T, R>): CPtr<R>
    return nil :: any
end

function CType.box<T, R>(self: CType<T, R>, val: R): Box
    return nil :: any
end

function CType.from<T, R>(self: CType<T, R>, ud: CTypes, offset: number?): R
    return nil :: any
end

function CType.into<T, R>(self: CType<T, R>, ud: CTypes, value: R, offset: number?)
    return nil :: any
end

function CType.arr<T, R>(self: CType<T, R>, len: number): CArr<R>
    return nil :: any
end

function CType.cast<T, R, F, I>(self: CType<T, R>, intoType: CTypes, from: F, into: I)
    return nil :: any
end

export type CPtr<T> = {
    size: number,
    inner: T?,
}

export type CArr<T> = {
    size: number,
    length: number,
    inner: {T}?,

    offset: (self: CArr<T>, offset: number) -> number,
    ptr: <K>(self: CArr<T>) -> CPtr<K>,
    box: <K>(self: CArr<T>, table: { K }) -> Box,
    from: <K>(self: CArr<T>, ud: any, offset: number?) -> { K },
    into: <K>(self: CArr<T>, ud: any, value: { K }, offset: number?) -> (),
}

type NumCType<T> = CType<T, number>

-- Fixed size Rust-style types --
export type u8 = NumCType<"u8">
export type u16 = NumCType<"u16">
export type u32 = NumCType<"u32">
export type u64 = NumCType<"u64">
export type u128 = NumCType<"u128">
export type i8 = NumCType<"i8">
export type i16 = NumCType<"i16">
export type i32 = NumCType<"i32">
export type i64 = NumCType<"i64">
export type i128 = NumCType<"i128">
export type f32 = NumCType<"f32">
export type f64 = NumCType<"f64">
export type usize = NumCType<"usize">
export type isize = NumCType<"isize">

-- Variable C-style types --
export type char = NumCType<"char">
export type float = NumCType<"float">
export type double = NumCType<"double">
export type uchar = NumCType<"uchar">
export type schar = NumCType<"schar">
export type short = NumCType<"short">
export type ushort = NumCType<"ushort">
export type int = NumCType<"int">
export type uint = NumCType<"uint">
export type long = NumCType<"long">
export type ulong = NumCType<"ulong">
export type longlong = NumCType<"longlong">
export type ulonglong = NumCType<"ulonglong">

export type CFn = {
    caller: () -> Callable
}

export type CTypes = 
    | u8
    | u16
    | u32
    | u64
    | u128
    | i8
    | i16
    | i32
    | i64
    | i128
    | f32
    | f64
    | usize
    | isize
    | char
    | float
    | double
    | uchar
    | schar
    | short
    | ushort
    | int 
    | uint
    | long
    | ulong
    | longlong
    | ulonglong

export type Ref = {
    deref: (self: Ref) -> Ref,
    offset: (self: Ref, offset: number) -> Ref,
    ref: (self: Ref) -> Ref, 
    isNullptr: (self: Ref) -> boolean,
}

export type Box = {
    size: number,
    
    zero: (self: Box) -> Box,
    leak: (self: Box, offset: number?) -> Ref,
    ref: (self: Box, offset: number?) -> Ref,
}

export type Library = {
    find: (sym: string) -> Ref,
}

export type Callable = {
    call: (...any) -> (), 
}

local ffi = {}

ffi.u8 = {} :: u8
ffi.nullptr = {} :: Ref

function ffi.box(size: number): Box
    return nil :: any
end

function ffi.open(path: string): Library
    return nil :: any
end

function ffi.ref(): Ref
    return nil :: any
end

function ffi.isInteger<T>(val: T): boolean
    return nil :: any
end

function ffi.fn<T>(args: { any }, ret: CPtr<T>): CFn
    return nil :: any
end
