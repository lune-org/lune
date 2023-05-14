# 🧑‍💻 Configuring VSCode and tooling for Lune

Lune puts developer experience first, and as such provides type definitions and configurations for several tools out of the box.

These steps assume you have already installed Lune and that it is available to run in the current directory.

## Luau LSP

1. Run `lune --setup` to generate Luau type definitions for your installed version of Lune
2. Verify that type definition files have been generated
3. Modify your VSCode settings, either by using the settings menu or in `settings.json`:

    ```json
    "luau-lsp.require.mode": "relativeToFile", // Set the require mode to work with Lune
    "luau-lsp.require.fileAliases": { // Add type definitions for Lune builtins
    	"@lune/fs": ".../.lune/.typedefs/x.y.z/fs.luau",
    	"@lune/net": ".../.lune/.typedefs/x.y.z/net.luau",
    	"@lune/...": "..."
    }
    ```

    _**NOTE:** If you already had a `.vscode/settings.json` file in your current directory the type definition files may have been added automatically!_
