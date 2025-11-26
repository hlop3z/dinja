/// Core library for dinja - pure Rust implementation
/// 
/// This crate contains the core logic, algorithms, and models
/// with no Python or FFI dependencies.

/// A simple hello world function
/// 
/// # Examples
/// 
/// ```
/// use dinja_core::hello;
/// 
/// let greeting = hello("World");
/// assert_eq!(greeting, "Hello, World!");
/// ```
pub fn hello(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello() {
        assert_eq!(hello("World"), "Hello, World!");
        assert_eq!(hello("Rust"), "Hello, Rust!");
    }
}

