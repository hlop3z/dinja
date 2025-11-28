//! Constants used throughout the renderer module

/// Script tags used for JavaScript execution in the runtime
pub(super) mod script_tags {
    pub const SETUP_CONTEXT: &str = "<setup_context>";
    pub const CLEANUP_RUNTIME: &str = "<cleanup_runtime>";
    pub const SETUP: &str = "<setup>";
    pub const RENDER: &str = "<render>";
    pub const HELPERS: &str = "<helpers>";
    pub const ENGINE: &str = "<engine>";
    pub const CHECK_ENGINE: &str = "<check_engine>";
    pub const WRAP_H_FUNCTION: &str = "<wrap_h_function>";
    pub const ENGINE_TO_STRING: &str = "<engine_to_string>";
    pub const CHECK_ENGINE_TO_STRING: &str = "<check_engine_to_string>";
    pub const CORE_ENGINE: &str = "<core_engine>";
    pub const CHECK_CORE_ENGINE: &str = "<check_core_engine>";
}

/// Static JavaScript file names
pub(super) mod static_files {
    pub const HELPERS_JS: &str = "helpers.js";
    pub const ENGINE_MIN_JS: &str = "engine.min.js";
    pub const ENGINE_TO_STRING_MIN_JS: &str = "engine_to_string.min.js";
    pub const CORE_JS: &str = "core.js";
}
