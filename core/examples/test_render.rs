use dinja_core::renderer::JsRenderer;
use dinja_core::transform::transform_tsx_to_js;
use std::path::PathBuf;

fn main() {
    let html = "<h1>Hello World</h1>\n<p>This is a test</p>";

    println!("Original HTML:");
    println!("{}\n", html);

    // Transform to JS
    let js = transform_tsx_to_js(html).expect("Failed to transform");
    println!("Transformed JavaScript:");
    println!("{}\n", js);

    // Try to render
    let static_dir = PathBuf::from("static");
    let renderer = JsRenderer::new(&static_dir).expect("Failed to create renderer");

    match renderer.render_transformed_component(&js, Some("{}"), None, None) {
        Ok(html) => {
            println!("Rendered HTML:");
            println!("{}", html);
        }
        Err(e) => {
            eprintln!("Render error: {:?}", e);
        }
    }
}
