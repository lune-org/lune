# Lune 🌙

[![CI](https://github.com/filiptibell/lune/actions/workflows/ci.yaml/badge.svg)](https://github.com/filiptibell/lune/actions/workflows/ci.yaml)
[![Release](https://github.com/filiptibell/lune/actions/workflows/release.yaml/badge.svg)](https://github.com/filiptibell/lune/actions/workflows/release.yaml)

A [Luau](https://luau-lang.org) script runner

---

🚀 Use the ergonomics and readability of Luau instead of shell scripts 🚀

[Full example & walkthrough](.lune/hello_lune.luau)

<!-- markdownlint-disable MD033 -->
<details>
<summary>Example translation from Bash to Luau</summary>

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

## ⚙️ Installation

### Using [Aftman](https://github.com/lpghatguy/aftman)

The preferred way of installing Lune.

This will add `lune` to an `aftman.toml` file in the
current directory, or create one if it does not exist.

```sh
$ aftman add filiptibell/lune
```

### From [GitHub Releases](https://github.com/filiptibell/lune/releases)

You can also download pre-built binaries for most
systems directly from the linked GitHub Releases page.

## ✏️ Writing Lune Scripts

First things first, check out the examples of how to write a script in the [.lune](.lune) folder! <br>
Lune has many useful built-in globals and APIs to use to interact with your system, and can do things
such as read/write files, run external programs, serialize & deserialize json, and much more.

## 🏃 Running Lune Scripts

When you've written a script with either a `.lua` or `.luau` extension, you can run it:

```sh
$ lune script-name
```

This will look for the script `script_name` in a few locations:

- The current directory
- The folder `lune` in the current directory, if it exists
- The folder `.lune` in the current directory, if it exists

If you don't want Lune to look in sub-directories you can provide a full
file path with the file extension included, instead of only the file name.
