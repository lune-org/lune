<!-- markdownlint-disable MD033 -->

# Lune 🌙

[![CI](https://github.com/filiptibell/lune/actions/workflows/ci.yaml/badge.svg)](https://github.com/filiptibell/lune/actions/workflows/ci.yaml)
[![Release](https://github.com/filiptibell/lune/actions/workflows/release.yaml/badge.svg)](https://github.com/filiptibell/lune/actions/workflows/release.yaml)

A [Luau](https://luau-lang.org) script runner

---

🚀 Use the ergonomics and readability of Luau instead of shell scripts 🚀

[Full example & walkthrough](.lune/hello_lune.luau)

## ⚙️ Installation

### Using [Aftman](https://github.com/lpghatguy/aftman)

The preferred way of installing Lune.

This will add `lune` to an `aftman.toml` file in the current directory, or create one if it does not exist.

```sh
$ aftman add filiptibell/lune
```

### From [GitHub Releases](https://github.com/filiptibell/lune/releases)

You can also download pre-built binaries for most systems directly from the linked GitHub Releases page.

## ✏️ Writing Lune Scripts

Check out the examples of how to write a script in the [.lune](.lune) folder !

<details>
<summary><b>🔎 Full list of APIs</b></summary>

### **`fs`** - Filesystem

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

### **`json`** - JSON

```lua
type json = {
	encode: (value: any, pretty: boolean?) -> string,
	decode: (encoded: string) -> any,
}
```

### **`process`** - Current process & child processes

```lua
type process = {
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
```

</details>

<details>
<summary><b>🔀 Example translation from Bash to Luau</b></summary>

**_Before:_**

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

**_After:_**

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
2. Set your definition files setting to include `luneTypes.d.luau`, an example can be found in the [.vscode](.vscode) folder in this repository

</details>

<details>
<summary>Selene</summary>

1. Use `lune --download-selene-types` to download Selene types (`lune.yml`) to the current directory
2. Use either `std = "roblox-lune"` or `std = "luau+lune"` in your `selene.toml` configuration file

</details>

**_NOTE:_** _It is highly recommended to add any type definition files to your `.gitignore` and to only download them using these commands, since this guarantees that you have type definitions compatible with your installed version of Lune._

</details>

## 🏃 Running Lune Scripts

When you've written a script with either a `.lua` or `.luau` extension, you can run it:

```sh
$ lune script-name
```

This will look for the script `script_name` in a few locations:

- The current directory
- The folder `lune` in the current directory, if it exists
- The folder `.lune` in the current directory, if it exists

If you don't want Lune to look in sub-directories you can provide a full file path with the file extension included, instead of only the file name.
