--[=[
	@type WriteOptions
	@within FS

	Options for filesystem APIs what write to files and/or directories.

	This is a dictionary that may contain one or more of the following values:

	* `overwrite` - If the target path should be overwritten or not, in the case that it already exists
]=]
export type WriteOptions = {
	overwrite: boolean?,
}

--[=[
	@class FS

	Built-in library for filesystem access

	### Example usage

	```lua
	local fs = require("@lune/fs")

	-- Reading a file
	local myTextFile: string = fs.readFile("myFileName.txt")

	-- Reading entries (files & dirs) in a directory
	for _, entryName in fs.readDir("myDirName") do
		if fs.isFile("myDirName/" .. entryName) then
			print("Found file " .. entryName)
		elseif fs.isDir("myDirName/" .. entryName) then
			print("Found subdirectory " .. entryName)
		end
	end
	```
]=]
export type FS = {
	--[=[
		@within FS
		@must_use

		Reads a file at `path`.

		An error will be thrown in the following situations:

		* `path` does not point to an existing file.
		* The current process lacks permissions to read the file.
		* The contents of the file cannot be read as a UTF-8 string.
		* Some other I/O error occurred.

		@param path The path to the file to read
		@return The contents of the file
	]=]
	readFile: (path: string) -> string,
	--[=[
		@within FS
		@must_use

		Reads entries in a directory at `path`.

		An error will be thrown in the following situations:

		* `path` does not point to an existing directory.
		* The current process lacks permissions to read the contents of the directory.
		* Some other I/O error occurred.

		@param path The directory path to search in
		@return A list of files & directories found
	]=]
	readDir: (path: string) -> { string },
	--[=[
		@within FS

		Writes to a file at `path`.

		An error will be thrown in the following situations:

		* The file's parent directory does not exist.
		* The current process lacks permissions to write to the file.
		* Some other I/O error occurred.

		@param path The path of the file
		@param contents The contents of the file
	]=]
	writeFile: (path: string, contents: string) -> (),
	--[=[
		@within FS

		Creates a directory and its parent directories if they are missing.

		An error will be thrown in the following situations:

		* `path` already points to an existing file or directory.
		* The current process lacks permissions to create the directory or its missing parents.
		* Some other I/O error occurred.

		@param path The directory to create
	]=]
	writeDir: (path: string) -> (),
	--[=[
		@within FS

		Removes a file.

		An error will be thrown in the following situations:

		* `path` does not point to an existing file.
		* The current process lacks permissions to remove the file.
		* Some other I/O error occurred.

		@param path The file to remove
	]=]
	removeFile: (path: string) -> (),
	--[=[
		@within FS

		Removes a directory and all of its contents.

		An error will be thrown in the following situations:

		* `path` is not an existing and empty directory.
		* The current process lacks permissions to remove the directory.
		* Some other I/O error occurred.

		@param path The directory to remove
	]=]
	removeDir: (path: string) -> (),
	--[=[
		@within FS
		@must_use

		Checks if a given path is a file.

		An error will be thrown in the following situations:

		* The current process lacks permissions to read at `path`.
		* Some other I/O error occurred.

		@param path The file path to check
		@return If the path is a file or not
	]=]
	isFile: (path: string) -> boolean,
	--[=[
		@within FS
		@must_use

		Checks if a given path is a directory.

		An error will be thrown in the following situations:

		* The current process lacks permissions to read at `path`.
		* Some other I/O error occurred.

		@param path The directory path to check
		@return If the path is a directory or not
	]=]
	isDir: (path: string) -> boolean,
	--[=[
		@within FS

		Moves a file or directory to a new path.

		Throws an error if a file or directory already exists at the target path.
		This can be bypassed by passing `true` as the third argument, or a dictionary of options.
		Refer to the documentation for `WriteOptions` for specific option keys and their values.

		An error will be thrown in the following situations:

		* The current process lacks permissions to read at `from` or write at `to`.
		* The new path exists on a different mount point.
		* Some other I/O error occurred.

		@param from The path to move from
		@param to The path to move to
		@param overwriteOrOptions Options for the target path, such as if should be overwritten if it already exists
	]=]
	move: (from: string, to: string, overwriteOrOptions: (boolean | WriteOptions)?) -> (),
}
