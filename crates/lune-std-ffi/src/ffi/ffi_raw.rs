// use core::ffi::c_void;
// use std::{convert, mem::transmute, ptr};

// This is raw data coming from outside.
// Users must convert it to a Lua value, reference, or box to use it.
// The biggest reason for providing this is to allow the user to
// decide whether to move the data to a heap that Lua can manage (box),
// move it directly to Lua's data, or think of it as a pointer.
// This will help you distinguish between safe operations and
// relatively insecure operations, and help ensure that as little
// data copy as possible occurs, while allowing you to do little restrictions.

pub struct FfiRaw(*const ());
