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

A great starting point and walkthrough of Lune can be found in [Hello, Lune](.lune/hello_lune.luau). <br />
More examples of how to write Lune scripts can be found in the [examples](.lune/examples/) folder.

<details>
<summary><b>🔎 List of APIs</b></summary>

`fs` - Filesystem <br />
`net` - Networking <br />
`process` - Current process & child processes <br />
`stdio` - Standard input / output & utility functions <br />
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

These steps assume you have already installed Lune and that it is available to run in the current directory.

<details>
<summary>Luau LSP</summary>

1. Run `lune --generate-luau-types` to generate a Luau type definitions file (`luneTypes.d.luau`) in the current directory
2. Run `lune --generate-docs-file` to generate a Luau LSP documentation file (`luneDocs.json`) in the current directory
3. Modify your VSCode settings, either by using the settings menu or in `settings.json`:

   ```json
   {
   	"luau-lsp.require.mode": "relativeToFile", // Set the require mode to work with Lune
   	"luau-lsp.types.definitionFiles": ["luneTypes.d.luau"], // Add type definitions for Lune globals
   	"luau-lsp.types.documentationFiles": ["luneDocs.json"] // Add documentation for Lune globals
   }
   ```

</details>

<details>

<summary>Selene</summary>

1. Run `lune --generate-selene-types` to generate a Selene type definitions file (`lune.yml`) in the current directory
2. Modify your Selene settings in `selene.toml`:

   ```yaml
   # Use this if Lune is the only thing you use Luau files with:
   std = "luau+lune"
   # OR use this if your project also contains Roblox-specific Luau code:
   std = "roblox+lune"
   # If you are also using the Luau type definitions file, it may cause issues, and can be safely ignored:
   exclude = ["luneTypes.d.luau"]
   ```

</details>
<br />

**_NOTE:_** _It is highly recommended to add any generated files to your `.gitignore` and to only generate them using these commands, since this guarantees that you have type definitions compatible with your installed version of Lune._

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
