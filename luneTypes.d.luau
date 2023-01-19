-- Lune v0.0.1

declare fs: {
	readFile: (path: string) -> string,
	readDir: (path: string) -> { string },
	writeFile: (path: string, contents: string) -> (),
	writeDir: (path: string) -> (),
	removeFile: (path: string) -> (),
	removeDir: (path: string) -> (),
	isFile: (path: string) -> boolean,
	isDir: (path: string) -> boolean,
}

declare json: {
	encode: (value: any, pretty: boolean?) -> string,
	decode: (encoded: string) -> any,
}

declare process: {
	getEnvVars: () -> { string },
	getEnvVar: (key: string) -> string?,
	setEnvVar: (key: string, value: string) -> (),
	exit: (code: number?) -> (),
	spawn: (program: string, params: { string }?) -> {
		ok: boolean,
		code: number,
		stdout: string,
		stderr: string,
	},
}
