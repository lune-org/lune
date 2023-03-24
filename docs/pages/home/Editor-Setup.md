# 🧑‍💻 Configuring VSCode and tooling for Lune

Lune puts developer experience first, and as such provides type definitions and configurations for several tools out of the box.

These steps assume you have already installed Lune and that it is available to run in the current directory.

## Luau LSP

1. Run `lune --generate-luau-types` to generate a Luau type definitions file (`luneTypes.d.luau`) in the current directory
2. Run `lune --generate-docs-file` to generate a Luau LSP documentation file (`luneDocs.json`) in the current directory
3. Modify your VSCode settings, either by using the settings menu or in `settings.json`:

    ```json
    "luau-lsp.require.mode": "relativeToFile", // Set the require mode to work with Lune
    "luau-lsp.types.definitionFiles": ["luneTypes.d.luau"], // Add type definitions for Lune globals
    "luau-lsp.types.documentationFiles": ["luneDocs.json"] // Add documentation for Lune globals
    ```

## Selene

1. Run `lune --generate-selene-types` to generate a Selene type definitions file (`lune.yml`) in the current directory
2. Modify your Selene settings in `selene.toml`:

    ```yaml
    # Use this if Lune is the only thing you use Luau files with:
    std = "luau+lune"
    # OR use this if your project also contains Roblox-specific Luau code:
    std = "roblox+lune"
    # If you are also using the Luau type definitions, they should be excluded:
    exclude = ["luneTypes.d.luau"]
    ```
