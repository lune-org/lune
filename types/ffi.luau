export type CType<T> = {
    size: number,
    signedness: boolean,

    ptr: <K>(self:  CType<T>) -> CPtr<K>,
    box: <K>(self: CType<T>, val: K) -> Box,
    from: <K>(self: CType<T>, ud: any, offset: number?) -> K,
    into: <K>(self: CType<T>, ud: any, value: K, offset: number?) -> (),
    arr: <K>(self: CType<T>, len: number) -> CArr<K>,
    -- FIXME: intoType is of type `CTypes`, but that leads to recursive types
    cast: <F, I>(self: CType<T>, intoType: any, from: F, into: I) -> ()
} & { ["__phantom"]: T }

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

-- Numeric types --
export type u8 = CType<"u8">
export type u16 = CType<"u16">
export type u32 = CType<"u32">
export type u64 = CType<"u64">
export type u128 = CType<"u128">
export type i8 = CType<"i8">
export type i16 = CType<"i16">
export type i32 = CType<"i32">
export type i64 = CType<"i64">
export type i128 = CType<"i128">
export type f32 = CType<"f32">
export type f64 = CType<"f64">
export type usize = CType<"usize">
export type isize = CType<"isize">

-- C types --
export type char = CType<"char">
export type float = CType<"float">
export type double = CType<"double">
export type uchar = CType<"uchar">
export type schar = CType<"schar">
export type short = CType<"short">
export type ushort = CType<"ushort">
export type int = CType<"int">
export type uint = CType<"uint">
export type long = CType<"long">
export type ulong = CType<"ulong">
export type longlong = CType<"longlong">
export type ulonglong = CType<"ulonglong">
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
