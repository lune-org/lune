-- Lune v0.0.2

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

declare net: {
	request: (config: string | {
		url: string,
		method: ("GET" | "POST" | "PUT" | "DELETE" | "HEAD" | "OPTIONS" | "PATCH")?,
		headers: { [string]: string }?,
		body: string?,
	}) -> {
		ok: boolean,
		statusCode: number,
		statusMessage: string,
		headers: { [string]: string },
		body: string,
	},
	jsonEncode: (value: any, pretty: boolean?) -> string,
	jsonDecode: (encoded: string) -> any,
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
