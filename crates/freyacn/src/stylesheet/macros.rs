/// Parse a CSS-like string into a [`Style`].
///
/// The input is a raw string literal containing property/value declarations
/// (no selector). Both `/* block */` and `// line` comments are stripped.
///
/// # Example
///
/// ```ignore
/// let style = style!(r#"
///     display: flex;
///     flex-direction: column;
///     padding: 20px;
///     gap: 8px;
///     width: calc(100% - 40px);
///     color: #222222;
///     background-color: #ffffff;
///     border-radius: 8px;
/// "#);
/// ```
#[macro_export]
macro_rules! style {
    ($css:literal) => {
        $crate::stylesheet::style_parser::parse_style($css)
    };
}

/// Parse a whitespace-separated list of Tailwind-like utility classes
/// into a [`Style`](crate::stylesheet::style::Style).
///
/// # Example
///
/// ```ignore
/// let style = class!("flex items-center p-4 bg-blue-500 rounded-lg shadow-md");
/// ```
#[macro_export]
macro_rules! class {
    ($classes:literal) => {
        $crate::stylesheet::class_parser::parse_classes($classes)
    };
}
