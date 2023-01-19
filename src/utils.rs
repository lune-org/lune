pub fn pretty_print_luau_error(e: &mlua::Error) {
    match e {
        mlua::Error::RuntimeError(e) => {
            eprintln!("{}", e);
        }
        mlua::Error::CallbackError { cause, traceback } => {
            pretty_print_luau_error(cause.as_ref());
            eprintln!("Traceback:");
            eprintln!("{}", traceback.strip_prefix("stack traceback:\n").unwrap());
        }
        mlua::Error::ToLuaConversionError { from, to, message } => {
            let msg = message
                .clone()
                .map(|m| format!("\nDetails:\n\t{m}"))
                .unwrap_or_else(|| "".to_string());
            eprintln!(
                "Failed to convert Rust type '{}' into Luau type '{}'!{}",
                from, to, msg
            )
        }
        mlua::Error::FromLuaConversionError { from, to, message } => {
            let msg = message
                .clone()
                .map(|m| format!("\nDetails:\n\t{m}"))
                .unwrap_or_else(|| "".to_string());
            eprintln!(
                "Failed to convert Luau type '{}' into Rust type '{}'!{}",
                from, to, msg
            )
        }
        e => eprintln!("{e}"),
    }
}
