--[=[
	@interface Box
	@within FFI

	Box is an untyped, sized memory area that Lua can manage.
    This area is safe within Lua. Operations have their boundaries checked.

    You can passing box as raw arguments or as pointer to outside.
    It also helps you handle data that Lua cannot handle. or you can reuse box to save cost from convertsion.
    Depending on the type, operations such as sum, mul, and mod may be implemented. See Types

    ```lua
    ffi.box(size)
    ```
	This is a dictionary that will contain the following values:

	* `readOnly` - If the target path is read-only or not
]=]

export type Box = {
	size: number,
    ref: (self: Box)->Ref,
}
export type BoxConstructor = (size: number)->Box

export type Type = {}

---! FIXME: better typing for PointerSize
export type PointerSize = number -- typeof(5) | typeof(8)

export type Arr<T> = {
    inner: T,
    size: number,
    ptr: (self: Arr<T>) -> any,
}

--[=[
	@interface Ptr
	@within FFI

]=]
---! FIXME: due to recursive type limition. hardcoded 6 depth. better idea?
export type Ptr<T> = {
    inner: T,
    size: PointerSize,
    ptr: (self: Ptr<T>)->PtrPtr<Ptr<T>>,
    arr: (self: Ptr<T>, size: number) -> Arr<Ptr<T>>,
}
export type PtrPtr<T> = {
    inner: T,
    size: PointerSize,
    ptr: (self: PtrPtr<T>)->PtrPtrPtr<PtrPtr<T>>,
    arr: (self: PtrPtr<T>, size: number) -> Arr<PtrPtr<T>>,
}
export type PtrPtrPtr<T> = {
    inner: T,
    size: PointerSize,
    ptr: (self: PtrPtrPtr<T>)->PtrPtrPtrPtr<PtrPtrPtr<T>>,
    arr: (self: PtrPtrPtr<T>, size: number) -> Arr<PtrPtrPtr<T>>,
}
export type PtrPtrPtrPtr<T> = {
    inner: T,
    size: PointerSize,
    ptr: (self: PtrPtrPtrPtr<T>)->PtrPtrPtrPtrPtr<PtrPtrPtrPtr<T>>,
    arr: (self: PtrPtrPtrPtr<T>, size: number) -> Arr<PtrPtrPtrPtr<T>>,
}
export type PtrPtrPtrPtrPtr<T> = {
    inner: T,
    size: PointerSize,
    ptr: (self: PtrPtrPtrPtrPtr<T>)->PtrPtrPtrPtrPtrPtr<PtrPtrPtrPtrPtr<T>>,
    arr: (self: PtrPtrPtrPtrPtr<T>, size: number) -> Arr<PtrPtrPtrPtrPtr<T>>,
}
export type PtrPtrPtrPtrPtrPtr<T> = {
    inner: T,
    size: PointerSize,
    ptr: (self: PtrPtrPtrPtrPtrPtr<T>)->any, -- Yes. At this point. more type is useless.
    arr: (self: PtrPtrPtrPtrPtrPtr<T>, size: number) -> Arr<PtrPtrPtrPtrPtrPtr<T>>,
}

