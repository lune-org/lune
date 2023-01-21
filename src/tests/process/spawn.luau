local result = process.spawn("ls", {
	"-a",
})

assert(result.ok, "Failed to spawn child process")

assert(result.stderr == "", "Stderr was not empty")
assert(result.stdout ~= "", "Stdout was empty")

assert(string.find(result.stdout, "Cargo.toml") ~= nil, "Missing Cargo.toml in output")
assert(string.find(result.stdout, ".gitignore") ~= nil, "Missing .gitignore in output")
