--[=[
	@class FFI
	
	Built-in library for foreign function interface

	### Example usage

	```lua
	```
]=]
local ffi = {}

--[=[
	@class C
	@within FFI

	Namespace for compile time sized c types.
]=]
local c = {}
ffi.c = c

--#region Data --

--[=[
	@class RefData
	
	A user manageable memory reference
]=]
export type RefData = {
	deref: (self: RefData) -> RefData,
	offset: (self: RefData, offset: number) -> RefData,
	ref: (self: RefData) -> RefData,
	isNull: (self: RefData) -> boolean,
}

--[=[
	@class BoxData
	
	A user manageable heap memory
]=]
export type BoxData = {
	--[=[
		@within BoxData
		@tag Field
		@field size

		Size of the box.
	]=]
	size: number,

	--[=[
		@within BoxData
		@tag Method
		@method zero

		Fill the box with zero.

		@return `Box` itself for convenience
	]=]
	zero: (self: BoxData) -> BoxData,
	--[=[
		@within BoxData
		@tag Method
		@method leak

		Create a reference of the box after leaking it.

		GC doesn't manage destruction after this action. You must free it later

		@return A reference of the box
	]=]
	leak: (self: BoxData, offset: number?) -> RefData,
	--[=[
		@within BoxData
		@tag Method
		@method ref

		Create a reference of the box.

		@return A reference of the box
	]=]
	ref: (self: BoxData, offset: number?) -> RefData,
}

--[=[
	@class LibData
	
	A dynamic opened library handle
]=]
export type LibData = {
	--[=[
		@within LibData
		@tag Method
		@method find

		Find a symbol from the dynamic library.

		@param sym The name of the symbol
		@return A `Ref` of the found symbol
	]=]
	find: (self: LibData, sym: string) -> RefData,
}

-- export type AppliedCallable = ()->()

--[=[
	@class CallableData
	@tag unsafe
	
	A callable external function
]=]
export type CallableData = (ret: (RefData|BoxData)?, ...RefData)->() & {
	-- apply: (self: Callable, args: Args)->AppliedCallable,
}

--[=[
	@class ClosureData
	
	A lua function wrapper for function pointer
]=]
export type ClosureData = {
	--[=[
		@within ClosureData
		@tag Method
		@method ref

		Create a reference of the closure. usually can be used for passing function pointer as argument

		@return A reference of the closure
	]=]
	ref: (self: ClosureData)->RefData,
}

--#endregion Data --

--#region C ABI Type Infos --

-- NOTE: T is a unique identifier for the `CType` and R is the closest Lua type.
export type CTypeInfo<T, R> = {
	size: number,
	signedness: boolean,

	-- subtype
	ptr: (self: CTypeInfo<T, R>) -> CPtrInfo<CTypeInfo<T, R>>,
	arr: (self: CTypeInfo<T, R>, len: number) -> CArrInfo<CTypeInfo<T, R>, R>,

	-- realize
	box: (self: CTypeInfo<T, R>, val: R) -> BoxData,
	readData: (self: CTypeInfo<T, R>, target: (RefData|BoxData), offset: number?) -> R,
	writeData: (self: CTypeInfo<T, R>, target: (RefData|BoxData), value: R, offset: number?) -> (),
	stringifyData: (self: CTypeInfo<T, R>, target: (RefData|BoxData), offset: number?) -> string,

	-- FIXME: recursive types; 'intoType' should be CTypes
	cast: (self: CTypeInfo<T, R>, intoType: any, fromData: (RefData|BoxData), intoData: (RefData|BoxData)) -> (),
} & { ["__phantom"]: T }
type NumCType<T> = CTypeInfo<T, (number|any)>

export type CPtrInfo<T> = {
	size: number,
	inner: T,

	-- subtype
	-- FIXME: recursive types; result 'any' should be CArrInfo<CPtrInfo<T>>
	arr: (self: CPtrInfo<T>, len: number) -> any,
	-- FIXME: recursive types; result 'any' should be CPtrInfo<CPtrInfo<T>>
	ptr: (self: CPtrInfo<T>) -> any,

	readRef: (self: CPtrInfo<T>, target: (RefData|BoxData), offset: number?) -> RefData,
	writeRef: (self: CPtrInfo<T>, target: (RefData|BoxData), value: (RefData|BoxData), offset: number?) -> (),
}

export type CArrInfo<T, R> = {
	size: number,
	length: number,
	inner: T,

	-- subtype
	ptr: (self: CArrInfo<T, R>) -> CPtrInfo<CArrInfo<T, R>>,

	-- realize
	box: (self: CArrInfo<T, R>, table: { T }) -> BoxData,
	readData: (self: CArrInfo<T, R>, target: (RefData|BoxData), offset: number?) -> { T },
	writeData: (self: CArrInfo<T, R>, target: (RefData|BoxData), value: { R }, target_offset: number?) -> (),
	copyData: (self: CArrInfo<T, R>, dst: (RefData|BoxData), src: (RefData|BoxData), dst_offset: number?, src_offset: number?) -> (),

	offset: (self: CArrInfo<T, R>, index: number) -> number,
}

export type CFnInfo = {
	callable: (self: CFnInfo, functionRef: RefData) -> CallableData,
	closure: (self: CFnInfo, (ret: RefData, ...RefData)->()) -> ClosureData,
}

export type CStructInfo = {
	size: number,

	arr: (self: CStructInfo, len: number) -> CArrInfo<CStructInfo, {any}>,
	ptr: (self: CStructInfo) -> CPtrInfo<CStructInfo>,

	box: (self: CStructInfo, table: { any }) -> BoxData,
	readData: (self: CStructInfo, target: (RefData|BoxData), offset: number?) -> { any },
	writeData: (self: CStructInfo, target: (RefData|BoxData), table: { any }, offset: number?) -> (),
	copyData: (self: CStructInfo, dst: (RefData|BoxData), src: (RefData|BoxData), dst_offset: number?, src_offset: number?) -> (),

	offset: (self: CStructInfo, index: number) -> number,
	field: (self: CStructInfo, index: number) -> CTypes,
}

--[=[
	@class CVoidInfo

	A type that represents c void. can only be used for the function return type.
]=]
export type CVoidInfo = {
	--[=[
		@within CVoidInfo
		@tag Method
		@method ptr

		Create a generic pointer type

		@return Generic pointer type
	]=]
	ptr: (self: CVoidInfo) -> CPtrInfo<CVoidInfo>,
}
c.void = {} :: CVoidInfo

--#endregion C ABI Type Infos --

--#region Fixed size Rust-style types --

--[=[
	@class u8
	@within FFI

	A 8-bit sized unsigned integer, Equivalent to `uint8_t` in `stdint`
]=]
ffi.u8 = {} :: u8
export type u8 = NumCType<"u8">
--[=[
	@class u16
	@within FFI

	A 16-bit sized unsigned integer, Equivalent to `uint16_t` in `stdint`
]=]
ffi.u16 = {} :: u16
export type u16 = NumCType<"u16">
--[=[
	@class u32
	@within FFI

	A 32-bit sized unsigned integer, Equivalent to `uint32_t` in `stdint`
]=]
ffi.u32 = {} :: u32
export type u32 = NumCType<"u32">
--[=[
	@class u64
	@within FFI

	A 64-bit sized unsigned integer, Equivalent to `uint64_t` in `stdint`
]=]
ffi.u64 = {} :: u64
export type u64 = NumCType<"u64">
--[=[
	@class u128
	@within FFI

	A 128-bit sized unsigned integer, Equivalent to `uint128_t` in `stdint`
]=]
ffi.u128 = {} :: u128
export type u128 = NumCType<"u128">
--[=[
	@class i8
	@within FFI

	A 8-bit sized signed integer, Equivalent to `int8_t` in `stdint`
]=]
ffi.i8 = {} :: i8
export type i8 = NumCType<"i8">
--[=[
	@class i16
	@within FFI

	A 16-bit sized signed integer, Equivalent to `int16_t` in `stdint`
]=]
ffi.i16 = {} :: i16
export type i16 = NumCType<"i16">
--[=[
	@class i32
	@within FFI

	A 32-bit sized signed integer, Equivalent to `int32_t` in `stdint`
]=]
ffi.i32 = {} :: i32
export type i32 = NumCType<"i32">
--[=[
	@class i64
	@within FFI

	A 64-bit sized signed integer, Equivalent to `int64_t` in `stdint`
]=]
ffi.i64 = {} :: i64
export type i64 = NumCType<"i64">
--[=[
	@class i128
	@within FFI

	A 128-bit sized signed integer, Equivalent to `int128_t` in `stdint`
]=]
ffi.i128 = {} :: i128
export type i128 = NumCType<"i128">
--[=[
	@class f32
	@within FFI

	A single-precision 32-bit sized floating-point, Almost always equivalent to `float` in C
]=]
ffi.f32 = {} :: f32
export type f32 = NumCType<"f32">
--[=[
	@class f64
	@within FFI

	A double-precision 64-bit sized floating-point, Almost always equivalent to `double` in C
]=]
ffi.f64 = {} :: f64
export type f64 = NumCType<"f64">
--[=[
	@class usize
	@within FFI

	A machine specific pointer sized unsigned integer,
]=]
ffi.usize = {} :: usize
export type usize = NumCType<"usize">
--[=[
	@class isize
	@within FFI

	A machine specific pointer sized signed integer,
]=]
ffi.isize = {} :: isize
export type isize = NumCType<"isize">

--#endregion Fixed size Rust-style types --

--#region Variable size C-style types --

c.char = {} :: char
export type char = NumCType<"char">
-- c.float = {} :: float
-- export type float = NumCType<"float">
-- c.double = {} :: double
-- export type double = NumCType<"double">
c.uchar = {} :: uchar
export type uchar = NumCType<"uchar">
c.schar = {} :: schar
export type schar = NumCType<"schar">
c.short = {} :: short
export type short = NumCType<"short">
c.ushort = {} :: ushort
export type ushort = NumCType<"ushort">
c.int = {} :: int
export type int = NumCType<"int">
c.uint = {} :: uint
export type uint = NumCType<"uint">
c.long = {} :: long
export type long = NumCType<"long">
c.ulong = {} :: ulong
export type ulong = NumCType<"ulong">
c.longlong = {} :: longlong
export type longlong = NumCType<"longlong">
c.ulonglong = {} :: ulonglong
export type ulonglong = NumCType<"ulonglong">

--#endregion Variable size C-style types --

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
	-- | float
	-- | double
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
	@within FFI

	Create a `Ref` with address 0.

	Can be used for receive a pointer from external function or pass it as an argument.

	@return A zero initialized Ref
]=]
function ffi.nullRef(): RefData
	return nil :: any
end

--[=[
	@within FFI

	Create a `Box` with specific size.

	@param size The size of the new box
	@return A allocated box
]=]
function ffi.box(size: number): BoxData
	return nil :: any
end

--[=[
	@within FFI

	Open a dynamic library.

	@param name The name of the target library
	@return A dynamic library handle
]=]
function ffi.open(name: string): LibData
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
