-- NOTE: T is a unique identifier for the `CType` and R is the closest Lua type.
export type CTypeInfo<T, R> = {
	size: number,
	signedness: boolean,

	-- subtype
	ptrInfo: (self: CTypeInfo<T, R>) -> CPtrInfo<CTypeInfo<T, R>>,
	arrInfo: (self: CTypeInfo<T, R>, len: number) -> CArrInfo<CTypeInfo<T, R>, R>,

	-- realize
	box: (self: CTypeInfo<T, R>, val: R) -> Box,
	readData: (self: CTypeInfo<T, R>, target: (Ref|Box), offset: number?) -> R,
	writeData: (self: CTypeInfo<T, R>, target: (Ref|Box), value: R, offset: number?) -> (),
	stringifyData: (self: CTypeInfo<T, R>, target: (Ref|Box), offset: number?) -> string,

	-- FIXME: recursive types; 'intoType' should be CTypes
	cast: (self: CTypeInfo<T, R>, intoType: any, fromData: (Ref|Box), intoData: (Ref|Box)) -> (),
} & { ["__phantom"]: T }

export type CPtrInfo<T> = {
	size: number,
	inner: T,

	-- subtype
	-- FIXME: recursive types; 'any' should be CPtrInfo
	arrInfo: (self: CPtrInfo<T>, len: number) -> any,
	ptrInfo: (self: CPtrInfo<T>) -> any,

	readRef: (self: CPtrInfo<T>, target: (Ref|Box), offset: number?) -> Ref,
	writeRef: (self: CPtrInfo<T>, target: (Ref|Box), value: (Ref|Box), offset: number?) -> (),
}

export type CArrInfo<T, R> = {
	size: number,
	length: number,
	inner: T,

	-- subtype
	ptrInfo: (self: CArrInfo<T, R>) -> CPtrInfo<T>,

	-- realize
	box: (self: CArrInfo<T, R>, table: { T }) -> Box,
	readData: (self: CArrInfo<T, R>, target: (Ref|Box), offset: number?) -> { T },
	writeData: (self: CArrInfo<T, R>, target: (Ref|Box), value: { T }, offset: number?) -> (),
	copyData: (self: CArrInfo<T, R>, dst: (Ref|Box), src: (Ref|Box), dst_offset: number?, src_offset: number?) -> (),

	offset: (self: CArrInfo<T, R>, offset: number) -> number,
}

export type CFnInfo = {
	callable: (self: CFnInfo, functionRef: Ref) -> Callable,
	closure: (self: CFnInfo, (...Ref)->()) -> (),
}

export type CStructInfo = {
	arrInfo: (self: CStructInfo, len: number) -> CArrInfo<CStructInfo, {any}>,
	ptrInfo: (self: CStructInfo) -> CPtrInfo<CStructInfo>,

	box: (self: CStructInfo, table: { any }) -> Box,
	readData: (self: CStructInfo, target: (Ref|Box), offset: number?) -> { any },
	writeData: (self: CStructInfo, target: (Ref|Box), table: { any }, offset: number?) -> (),
}

type NumCType<T> = CTypeInfo<T, (number|any)>

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
	| CArrInfo<CTypes, any>
	| CPtrInfo<CTypes>
	| CFnInfo
	| CStructInfo

export type Ref = {
	deref: (self: Ref) -> Ref,
	offset: (self: Ref, offset: number) -> Ref,
	ref: (self: Ref) -> Ref,
	isNull: (self: Ref) -> boolean,
}

export type Box = {
	size: number,

	zero: (self: Box) -> Box,
	leak: (self: Box, offset: number?) -> Ref,
	ref: (self: Box, offset: number?) -> Ref,
}

export type Lib = {
	find: (self: Lib, sym: string) -> Ref,
}

export type Callable = {
	call: (self: Callable, result: (Ref | Box), ...(Ref | Box))->();
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

function ffi.nullRef(): Ref
	return nil :: any
end

function ffi.box(size: number): Box
	return nil :: any
end

function ffi.open(name: string): Lib
	return nil :: any
end

function ffi.uninitRef(): Ref
	return nil :: any
end

function ffi.isInteger<T>(val: T): boolean
	return nil :: any
end

function ffi.fnInfo(args: { CTypes }, ret: CTypes): CFnInfo
	return nil :: any
end

function ffi.structInfo(inner: { CTypes }): CStructInfo
	return nil :: any
end

return ffi
