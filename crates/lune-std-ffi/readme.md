TODO: rewrite docs

# Raw

Data received from external. You can move this data into a box, use it as a ref, or change it directly to a Lua value.
The raw data is not on Lua's heap.

Raw:toRef()
Convert data into ref. it allocate new lua userdata

Raw:toBox()
Convert data into box. it allocate new lua userdata

Raw:intoBox()
Raw:intoRef()

See type:fromRaw()

# Box

`ffi.box(size)`

Create new userdata with sized by `size` argument. Box is untyped, and have no ABI information. You can write some data into box with `type`

All operation with box will boundary checked. GC will free heap well.

일반적으로 포인터를 넘겨주기 위해서 사용됩니다. 박스의 공간은 ref 할 수 있으며. 함수를 수행한 후 루아에서 읽어볼 수 있습니다.

## :zero()
박스를 0 으로 채워넣습니다. 기본적으로 박스는 초기화될 때 0 으로 채워지기 때문에 박스를 다시 0 으로 초기화하고 싶을 경우에 사용하십시오.

## :copy(targetbox,size,offset?=0,targetoffset?=0)
박스 안의 값을 다른 박스에 복사합니다. 바운더리가 확인되어지므로 안전합니다.

## .size
이 박스의 크기입니다.

## :ref(offset?=0) => ref
이 박스를 참조합니다. 참조가 살아있는 동안 박스는 수거되지 않습니다. 일반적으로 외부의 함수에 포인터를 넘겨주기 위해서 사용됩니다.

## more stuffs (not planned at this time)

ref=>buffer conversion, or bit/byte related?

# Ref (Unsafe)

바운더리를 처리하지 않는 포인터입니다. 외부에서 받은 포인터, 또는 박스로부터 만들어진 포인터입니다.
ref 는 바운더리를 검사하지 않으므로 안전하지 않습니다.

## :offset(bytes)

이 ref 와 상대적인 위치에 있는 ref 를 구합니다.

## :writefromRef()
다른 ref 안의 값을 읽어와 이 ref 안에 씁니다. 아래와 비슷한 연산을 합니다
```c
int a = 100,b;
```

## :writefromBox()
box 값을 읽어와서 쓰기

# Type

`type` is abstract class that helps encoding data into `box` or decode data from `box`

## :toBox(luavalue)
Convert lua value to box. box will sized with `type.size`

## :fromBox(box,offset?=0)
Read data from box, and convert into lua value.
Boundary will checked

## :intoBox(luavalue,box,offset?=0)
Convert lua value, and write into box
Boundary will checked

## :fromRef(ref,offset?=0)
포인터가 가르키는 곳의 데이터를 읽어서 루아의 데이터로 변환합니다.

## :intoRef(luavalue,ref,offset?=0)
포인터가 가르키는 곳에 데이터를 작성합니다.

## :fromRaw(raw,offset?=0)


## :ptr() -> Ptr
Get pointer type

## :arr(len) -> Arr
Get array type

## .size

Byte size of this type. you can initialize box with

## :cast(box,type) TODO

# Ptr
Pointer type of some type.

Ptr is not data converter. It only works for type hint of `struct` or `fn`

## .inner
Inner type

## .size
Size of `usize`

:ptr()
:arr()

## Arr

## Void

`ffi.void`

Zero sized type.

## Fn
Prototype type of some function. converts lua function into native function pointer or native function pointer into lua function.

`ffi.fn({ type }, type) -> fn`

:toLua( ref ) -> luafunction
:toBox( luafunction ) -> ref

> TODO: rust, and another ABI support
