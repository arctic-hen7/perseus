use std::env;

/// Determines the engine operation to be performed by examining environment variables (set automatically by the CLI as appropriate).
pub fn get_op() -> Option<EngineOperation> {
    let var = match env::var("PERSEUS_ENGINE_OPERATION").ok() {
        Some(var) => var,
        None => {
            return {
                // The only typical use of a release-built binary is as a server, in which case we shouldn't need to specify this environment variable
                // So, in production, we take the server as the default
                // If a user wants a builder though, they can just set the environment variable
                // TODO Document this!
                if cfg!(debug_assertions) {
                    None
                } else {
                    Some(EngineOperation::Serve)
                }
            };
        }
    };

    match var.as_str() {
        "serve" => Some(EngineOperation::Serve),
        "build" => Some(EngineOperation::Build),
        "export" => Some(EngineOperation::Export),
        "export_error_page" => Some(EngineOperation::ExportErrorPage),
        "tinker" => Some(EngineOperation::Tinker),
        _ => {
            if cfg!(debug_assertions) {
                None
            } else {
                Some(EngineOperation::Serve)
            }
        }
    }
}

/// A representation of the server-side engine operations that can be performed.
pub enum EngineOperation {
    /// Run the server for the app. This assumes the app has already been built.
    Serve,
    /// Build the app. This process involves statically generating HTML and the like to be sent to the client.
    Build,
    /// Export the app by building it and also creating a file layout suitable for static file serving.
    Export,
    /// Export a single error page to a single file.
    ExportErrorPage,
    /// Run the tinker plugin actions.
    Tinker,
}
