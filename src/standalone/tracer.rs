/*
    TODO: Implement tracing of requires here

    Rough steps / outline:

    1. Create a new tracer struct using a main entrypoint script path
    2. Some kind of discovery mechanism that goes through all require chains (failing on recursive ones)
       2a. Conversion of script-relative paths to cwd-relative paths + normalization
       2b. Cache all found files in a map of file path -> file contents
       2c. Prepend some kind of symbol to paths that can tell our runtime `require` function that it
           should look up a bundled/standalone script, a good symbol here is probably a dollar sign ($)
    3. ???
    4. Profit
*/
