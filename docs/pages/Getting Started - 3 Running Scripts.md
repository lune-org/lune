<!-- markdownlint-disable MD033 -->

# 🏃 Running Lune Scripts

After you've written a script file, for example `script-name.luau`, you can run it:

```sh
lune script-name
```

This will look for the file `script-name.luau`**_<sup>[1]</sup>_** in a few locations:

-   The current directory
-   The folder `lune` in the current directory, if it exists
-   The folder `.lune` in the current directory, if it exists

## 🎛️ Passing Command-Line Arguments

Arguments can be passed to a Lune script directory from the command line when running it:

```sh
lune script-name arg1 arg2 "argument three"
```

These arguments will then be available in your script using `process.args`:

```lua
print(process.args)
--> { "arg1", "arg2", "argument three" }
```

## 💭 Additional Commands

```sh
lune --list
```

Lists all scripts found in `lune` or `.lune` directories, including any top-level description comments. <br />
Lune description comments are always written at the top of a file and start with a lua-style comment arrow (`-->`).

```sh
lune -
```

Runs a script passed to Lune using stdin. Occasionally useful for running scripts piped to Lune from external sources.

---

**_<sup>[1]</sup>_** _Lune also supports files with the `.lua` extension but using the `.luau` extension is highly recommended. Additionally, if you don't want Lune to look in sub-directories or try to find files with `.lua` / `.luau` extensions at all, you can provide an absolute file path. This will disable all file path parsing and checks, and just run the file directly._
