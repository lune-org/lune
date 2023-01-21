-- Write two inner dirs in the bir dir, a parent and a child

fs.writeDir("bin/test_dir/test_inner")

-- Make sure dir checks succeed but file
-- checks fail for all levels of dirs

assert(fs.isDir("bin"), "Dir root isDir check failed")
assert(fs.isDir("bin/test_dir"), "Dir outer isDir check failed")
assert(fs.isDir("bin/test_dir/test_inner"), "Dir inner isDir check failed")

assert(not fs.isFile("bin"), "Dir root isFile check failed")
assert(not fs.isFile("bin/test_dir"), "Dir outer isFile check failed")
assert(not fs.isFile("bin/test_dir/test_inner"), "Dir inner isFile check failed")

-- Remove the created parent and child dirs and
-- make sure the APIs say they no longer exist

fs.removeDir("bin/test_dir/")

assert(not fs.isDir("bin/test_dir"), "After removal isDir check failed")
assert(not fs.isFile("bin/test_dir"), "After removal isFile check failed")
