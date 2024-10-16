-- NOTE: T is a unique identifier for the `CType` and R is the closest Lua type.
export type CType<T, R> = {
	size: number,
	signedness: boolean,

	ptr: (self: CType<T, R>) -> CPtr<R>,
	box: (self: CType<T, R>, val: R) -> Box,
	-- FIXME: recursive types; ud should be CTypes
	from: (self: CType<T, R>, ud: any, offset: number?) -> R,
	into: (self: CType<T, R>, ud: any, value: R, offset: number?) -> (),
	arr: (self: CType<T, R>, len: number) -> CArr<R>,
	-- FIXME: recursive types; intoType should be CTypes
	cast: <F, I>(self: CType<T, R>, intoType: any, from: F, into: I) -> (),
} & { ["__phantom"]: T }

export type CPtr<T> = {
	size: number,
	inner: T?,
}

export type CArr<T> = {
	size: number,
	length: number,
	inner: { T }?,

	offset: (self: CArr<T>, offset: number) -> number,
	ptr: (self: CArr<T>) -> CPtr<{ T }>,
	box: (self: CArr<T>, table: { T }) -> Box,
	-- FIXME: recursive types; ud should be CTypes
	from: (self: CArr<T>, ud: any, offset: number?) -> { T },
	into: (self: CArr<T>, ud: any, value: { T }, offset: number?) -> (),
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

-- Variable size C-style types --
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
	caller: (self: CFn, fnPtr: Ref) -> Callable,
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
	find: (self: Library, sym: string) -> Ref,
}

export type Callable = {
	call: (self: Callable, retPtr: Ref, ...Box) -> (),
}

local ffi = {}

ffi.u8 = (nil :: unknown) :: u8
ffi.u16 = {} :: u16
ffi.u32 = {} :: u32
ffi.u64 = {} :: u64
ffi.u128 = {} :: u128
ffi.i8 = {} :: i8
ffi.i16 = {} :: i16
ffi.i32 = {} :: i32
ffi.i64 = {} :: i64
ffi.i128 = {} :: i128
ffi.f32 = {} :: f32
ffi.f64 = {} :: f64
ffi.usize = {} :: usize
ffi.isize = {} :: isize

ffi.char = {} :: char
ffi.float = {} :: float
ffi.double = {} :: double
ffi.uchar = {} :: uchar
ffi.schar = {} :: schar
ffi.short = {} :: short
ffi.ushort = {} :: ushort
ffi.int = {} :: int
ffi.uint = {} :: uint
ffi.long = {} :: long
ffi.ulong = {} :: ulong
ffi.longlong = {} :: longlong
ffi.ulonglong = {} :: ulonglong

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

function ffi.fn<T>(args: { CTypes }, ret: CTypes): CFn
	return nil :: any
end

return ffi
