local ffi = require("@lune/ffi")

local floatBox = ffi.f32:box(1.2)
local intBox = ffi.box(ffi.i32.size)

ffi.f32:cast(ffi.i32, floatBox, intBox)

local castedInt = ffi.i32:readData(intBox)

assert(
	castedInt == 1 and ffi.isInteger(castedInt),
	"castedInt == 1 and ffi.isInteger(castedInt) assersion failed"
)
