<!-- markdownlint-disable MD033 -->

# Lune 🌙

[![CI](https://github.com/filiptibell/lune/actions/workflows/ci.yaml/badge.svg)](https://github.com/filiptibell/lune/actions/workflows/ci.yaml)
[![Release](https://github.com/filiptibell/lune/actions/workflows/release.yaml/badge.svg)](https://github.com/filiptibell/lune/actions/workflows/release.yaml)

A [Luau](https://luau-lang.org) script runner

---

🚀 Use the ergonomics and readability of Luau instead of shell scripts 🚀

[Full example & walkthrough](.lune/hello_lune.luau)

## ⚙️ Installation

The preferred way of installing Lune is using [Aftman](https://github.com/lpghatguy/aftman).

This will add `lune` to an `aftman.toml` file in the current directory, or create one if it does not exist:

```sh
aftman add filiptibell/lune
```

You can also download pre-built binaries for most systems directly from the GitHub Releases page.

## ✏️ Writing Lune Scripts

Check out the examples of how to write a script in the [.lune](.lune) folder !

<details>
<summary><b>🔎 Full list of APIs</b></summary>

<details>
<summary><b>console</b> - Logging & formatting</summary>

```lua
type console = {
	resetColor: () -> (),
	setColor: (color: "black" | "red" | "green" | "yellow" | "blue" | "purple" | "cyan" | "white") -> (),
	resetStyle: () -> (),
	setStyle: (color: "bold" | "dim") -> (),
	format: (...any) -> (string),
	log: (...any) -> (),
	info: (...any) -> (),
	warn: (...any) -> (),
	error: (...any) -> (),
}
```

</details>

<details>
<summary><b>fs</b> - Filesystem</summary>

```lua
type fs = {
	readFile: (path: string) -> string,
	readDir: (path: string) -> { string },
	writeFile: (path: string, contents: string) -> (),
	writeDir: (path: string) -> (),
	removeFile: (path: string) -> (),
	removeDir: (path: string) -> (),
	isFile: (path: string) -> boolean,
	isDir: (path: string) -> boolean,
}
```

</details>

<details>
<summary><b>net</b> - Networking</summary>

```lua
type net = {
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
```

</details>

<details>
<summary><b>process</b> - Current process & child processes</summary>

```lua
type process = {
	args: { string },
	env: { [string]: string? },
	exit: (code: number?) -> (),
	spawn: (program: string, params: { string }?) -> {
		ok: boolean,
		code: number,
		stdout: string,
		stderr: string,
	},
}
```

</details>

<details>
<summary><b>task</b> - Task scheduler & thread spawning</summary>

```lua
type task = {
	cancel: (thread: thread) -> (),
	defer: (functionOrThread: thread | (...any) -> (...any)) -> thread,
	delay: (duration: number?, functionOrThread: thread | (...any) -> (...any)) -> thread,
	spawn: (functionOrThread: thread | (...any) -> (...any)) -> thread,
	wait: (duration: number?) -> (number),
}
```

</details>

</details>

<details>
<summary><b>🔀 Example translation from Bash</b></summary>

```bash
#!/bin/bash
VALID=true
COUNT=1
while [ $VALID ]
do
    echo $COUNT
    if [ $COUNT -eq 5 ];
    then
        break
    fi
    ((COUNT++))
done
```

**_With Lune & Luau:_**

```lua
local valid = true
local count = 1
while valid do
    print(count)
    if count == 5 then
        break
    end
    count += 1
end
```

</details>

<details>
<summary><b>🧑‍💻 Configuring VSCode for Lune</b></summary>

Lune puts developer experience first, and as such provides type definitions and configurations for several tools out of the box.

<details>
<summary>Luau LSP</summary>

1. Use `lune --download-luau-types` to download Luau types (`luneTypes.d.luau`) to the current directory
2. Set your definition files setting to include `luneTypes.d.luau`
3. Set the require mode setting to `relativeToFile`

En example of these settings can be found in the [.vscode](.vscode) folder in this repository

</details>

<details>

<summary>Selene</summary>

1. Use `lune --download-selene-types` to download Selene types (`lune.yml`) to the current directory
2. Use either `std = "luau+lune"`, or `std = "roblox+lune"` if your project also contains Roblox-specific code, in your `selene.toml` configuration file

</details>
<br>

**_NOTE:_** _It is highly recommended to add any type definition files to your `.gitignore` and to only download them using these commands, since this guarantees that you have type definitions compatible with your installed version of Lune._

</details>

## 🏃 Running Lune Scripts

After you've written a script file, for example `script-name.luau`, you can run it:

```sh
lune script-name
```

This will look for the file `script-name.luau` in a few locations:

- The current directory
- The folder `lune` in the current directory, if it exists
- The folder `.lune` in the current directory, if it exists

If you don't want Lune to look in sub-directories you can provide a full file path with the file extension included, instead of only the file name. <br>

---

**_NOTE:_** _Lune also supports files with the `.lua` extension but using the `.luau` extension is highly recommended._
