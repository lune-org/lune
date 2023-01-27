<!-- markdownlint-disable MD033 -->
<!-- markdownlint-disable MD041 -->

<div align="center">
	<h1> Lune 🌙 </h1>
	<div>
		<a href="https://crates.io/crates/lune"><img src="https://img.shields.io/crates/v/lune.svg?label=Version" alt="Current Lune library version" />
		<a href="https://github.com/filiptibell/lune/actions"><img src="https://shields.io/endpoint?url=https://badges.readysetplay.io/workflow/filiptibell/lune/ci.yaml" alt="CI status" />
		<a href="https://github.com/filiptibell/lune/actions"><img src="https://shields.io/endpoint?url=https://badges.readysetplay.io/workflow/filiptibell/lune/release.yaml" alt="Release status" />
		<a href="https://github.com/filiptibell/lune/blob/main/LICENSE.txt"><img src="https://img.shields.io/github/license/filiptibell/lune.svg?label=License&color=informational" alt="Current Lune library version" />
	</div>
	<br /> A standalone <a href="https://luau-lang.org">Luau</a> script runner
	<br /> 🚀 Use the ergonomics and readability of Luau for your shell scripts 🚀
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

Check out the examples on how to write a script in the [.lune](.lune) folder ! <br />
A great starting point and walkthrough of Lune can be found in the [Hello, Lune](.lune/hello_lune.luau) example.

<details>
<summary><b>🔎 List of APIs</b></summary>

`console` - Logging & formatting <br />
`fs` - Filesystem <br />
`net` - Networking <br />
`process` - Current process & child processes <br />
`task` - Task scheduler & thread spawning <br />

Documentation for individual members and types can be found using your editor of choice and [Luau LSP](https://github.com/JohnnyMorganz/luau-lsp).

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

1. Set the require mode setting to `relativeToFile`
2. Use `lune --download-luau-types` to download Luau types (`luneTypes.d.luau`) to the current directory
3. Set your definition files setting to include `luneTypes.d.luau`
4. Generate the documentation file using `lune --generate-docs-file`
   - NOTE: This is a temporary solution and a docs file separate from type definitions will not be necessary in the future
5. Set your documentation files setting to include `luneDocs.json`

An example of these settings can be found in the [.vscode](.vscode) folder in this repository

</details>

<details>

<summary>Selene</summary>

1. Use `lune --download-selene-types` to download Selene types (`lune.yml`) to the current directory
2. Use either `std = "luau+lune"`, or `std = "roblox+lune"` if your project also contains Roblox-specific code, in your `selene.toml` configuration file

</details>
<br />

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

If you don't want Lune to look in sub-directories you can provide a full file path with the file extension included, instead of only the file name. <br />

## 💭 Additional Commands

```sh
lune --list
```

Lists all scripts found in `lune` or `.lune` directories, including any top-level description comments. <br />
Lune description comments are always written at the top of a file and start with a lua-style comment arrow (`-->`).

---

**_NOTE:_** _Lune also supports files with the `.lua` extension but using the `.luau` extension is highly recommended._
