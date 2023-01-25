<!-- markdownlint-disable MD033 -->
<!-- markdownlint-disable MD041 -->

<div align="center">
	<h1> Lune 🌙 </h1>

<div align="center">
	<a href="https://crates.io/crates/lune"><img src="https://img.shields.io/crates/v/lune.svg?label=Version" alt="Current Lune library version" /></a>
	<a href="https://github.com/filiptibell/lune/actions"><img src="https://shields.io/endpoint?url=https://badges.readysetplay.io/workflow/filiptibell/lune/ci.yaml" alt="CI status" /></a>
	<a href="https://github.com/filiptibell/lune/actions"><img src="https://shields.io/endpoint?url=https://badges.readysetplay.io/workflow/filiptibell/lune/release.yaml" alt="Release status" /></a>
	<a href="https://github.com/filiptibell/lune/blob/main/LICENSE.txt"><img src="https://img.shields.io/github/license/filiptibell/lune.svg?label=License&color=informational" alt="Current Lune library version" /></a>
</div>

<br />

A standalone <a href="https://luau-lang.org">Luau</a> script runner

🚀 Use the ergonomics and readability of Luau for your shell scripts 🚀

</div>

<hr />

## ⚙️ Installation

The preferred way of installing Lune is using [Aftman](https://github.com/lpghatguy/aftman).

This will add `lune` to an `aftman.toml` file in the current directory, or create one if it does not exist:

```sh
aftman add filiptibell/lune
```

You can also download pre-built binaries for most systems directly from the GitHub Releases page.

## ✏️ Writing Lune Scripts

Check out the examples on how to write a script in the [.lune](.lune) folder ! <br>
A great starting point and walkthrough of Lune can be found in the [Hello, Lune](.lune/hello_lune.luau) example.

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
	defer: <T...>(functionOrThread: thread | (T...) -> (...any), T...) -> thread,
	delay: <T...>(duration: number?, functionOrThread: thread | (T...) -> (...any), T...) -> thread,
	spawn: <T...>(functionOrThread: thread | (T...) -> (...any), T...) -> thread,
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

An example of these settings can be found in the [.vscode](.vscode) folder in this repository

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

## 💭 Additional Commands

```sh
lune --list
```

Lists all scripts found in `lune` or `.lune` directories, including description comments if they exist. <br>
If both `lune` and `.lune` directories exist, only the former will have its scripts listed, which is consistent with the behavior of running scripts.

---

**_NOTE:_** _Lune also supports files with the `.lua` extension but using the `.luau` extension is highly recommended._
