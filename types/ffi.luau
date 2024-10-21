-- NOTE: T is a unique identifier for the `CType` and R is the closest Lua type.
export type CTypeInfo<T, R> = {
	size: number,
	signedness: boolean,

	-- subtype
	ptr: (self: CTypeInfo<T, R>) -> CPtrInfo<CTypeInfo<T, R>>,
	arr: (self: CTypeInfo<T, R>, len: number) -> CArrInfo<CTypeInfo<T, R>, R>,

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
	-- FIXME: recursive types; result 'any' should be CArrInfo<CPtrInfo<T>>
	arr: (self: CPtrInfo<T>, len: number) -> any,
	-- FIXME: recursive types; result 'any' should be CPtrInfo<CPtrInfo<T>>
	ptr: (self: CPtrInfo<T>) -> any,

	readRef: (self: CPtrInfo<T>, target: (Ref|Box), offset: number?) -> Ref,
	writeRef: (self: CPtrInfo<T>, target: (Ref|Box), value: (Ref|Box), offset: number?) -> (),
}

export type CArrInfo<T, R> = {
	size: number,
	length: number,
	inner: T,

	-- subtype
	ptr: (self: CArrInfo<T, R>) -> CPtrInfo<CArrInfo<T, R>>,

	-- realize
	box: (self: CArrInfo<T, R>, table: { T }) -> Box,
	readData: (self: CArrInfo<T, R>, target: (Ref|Box), offset: number?) -> { T },
	writeData: (self: CArrInfo<T, R>, target: (Ref|Box), value: { R }, target_offset: number?) -> (),
	copyData: (self: CArrInfo<T, R>, dst: (Ref|Box), src: (Ref|Box), dst_offset: number?, src_offset: number?) -> (),

	offset: (self: CArrInfo<T, R>, index: number) -> number,
}

export type CFnInfo = {
	callable: (self: CFnInfo, functionRef: Ref) -> Callable,
	closure: (self: CFnInfo, (ret: Ref, ...Ref)->()) -> Closure,
}

export type CStructInfo = {
	size: number,

	arr: (self: CStructInfo, len: number) -> CArrInfo<CStructInfo, {any}>,
	ptr: (self: CStructInfo) -> CPtrInfo<CStructInfo>,

	box: (self: CStructInfo, table: { any }) -> Box,
	readData: (self: CStructInfo, target: (Ref|Box), offset: number?) -> { any },
	writeData: (self: CStructInfo, target: (Ref|Box), table: { any }, offset: number?) -> (),
	copyData: (self: CStructInfo, dst: (Ref|Box), src: (Ref|Box), dst_offset: number?, src_offset: number?) -> (),

	offset: (self: CStructInfo, index: number) -> number,
	field: (self: CStructInfo, index: number) -> CTypes,
}

export type CVoidInfo = {
	ptr: (self: CVoidInfo) -> CPtrInfo<CVoidInfo>,
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
	| CVoidInfo

export type Ref = {
	deref: (self: Ref) -> Ref,
	offset: (self: Ref, offset: number) -> Ref,
	ref: (self: Ref) -> Ref,
	isNull: (self: Ref) -> boolean,
}

--[=[
	@class Box
	
	A user manageable heap memory
]=]
export type Box = {
	--[=[
		@within Box
		@tag Field
		@field size

		Size of the box.
	]=]
	size: number,

	--[=[
		@within Box
		@tag Method
		@method zero

		Fill the box with zero.

		@return `Box` itself for convenience
	]=]
	zero: (self: Box) -> Box,
	--[=[
		@within Box
		@tag Method
		@method leak

		Create a reference of the box after leaking it.

		GC doesn't manage destruction after this action. You must free it later

		@return A reference of the box
	]=]
	leak: (self: Box, offset: number?) -> Ref,
	--[=[
		@within Box
		@tag Method
		@method ref

		Create a reference of the box.

		@return A reference of the box
	]=]
	ref: (self: Box, offset: number?) -> Ref,
}

--[=[
	@class Lib
	
	A dynamic opened library handle
]=]
export type Lib = {
	--[=[
		@within Lib
		@tag Method
		@method find

		Find a symbol from the dynamic library.

		@param sym The name of the symbol
		@return A `Ref` of the found symbol
	]=]
	find: (self: Lib, sym: string) -> Ref,
}

-- export type AppliedCallable = ()->()

--[=[
	@class Callable
	@tag unsafe
	
	A callable external function
]=]
export type Callable = (ret: (Ref|Box)?, ...Ref)->() & {
	-- apply: (self: Callable, args: Args)->AppliedCallable,
}

export type Closure = {
	ref: (self: Closure)->Ref,
}

--[=[
	@class C
	@within FFI

	Namespace for compile time sized c types.
]=]
local c = {}

c.char = {} :: char
c.float = {} :: float
c.double = {} :: double
c.uchar = {} :: uchar
c.schar = {} :: schar
c.short = {} :: short
c.ushort = {} :: ushort
c.int = {} :: int
c.uint = {} :: uint
c.long = {} :: long
c.ulong = {} :: ulong
c.longlong = {} :: longlong
c.ulonglong = {} :: ulonglong

c.void = {} :: CVoidInfo

--[=[
	@within C

	Create a function signature type information.

	@param args An array of CTypes represents the arguments of the function
	@param ret The return type of the function
	@return A function signature type information
]=]
function c.fn(args: { CTypes }, ret: CTypes): CFnInfo
	return nil :: any
end

--[=[
	@within C

	Create a struct type information.

	@param fields An array of CTypes represents the fields of the struct 
	@return A struct type information
]=]
function c.struct(fields: { CTypes }): CStructInfo
	return nil :: any
end

--[=[
	@class FFI
	
	Built-in library for foreign function interface

	### Example usage

	```lua
	```
]=]
local ffi = {}

ffi.c = c

ffi.u8 = {} :: u8
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

--[=[
	@within FFI

	Create a `Ref` with address 0.

	Can be used for receive a pointer from external function or pass it as an argument.

	@return A zero initialized Ref
]=]
function ffi.nullRef(): Ref
	return nil :: any
end

--[=[
	@within FFI

	Create a `Box` with specific size.

	@param size The size of the new box
	@return A allocated box
]=]
function ffi.box(size: number): Box
	return nil :: any
end

--[=[
	@within FFI

	Open a dynamic library.

	@param name The name of the target library
	@return A dynamic library handle
]=]
function ffi.open(name: string): Lib
	return nil :: any
end

--[=[
	@within FFI

	Return `true` if the second argument is an integer (i32)

	@param val A lua value to check
	@return Whether val is an integer or not
]=]
function ffi.isInteger<T>(val: T): boolean
	return nil :: any
end


return ffi
