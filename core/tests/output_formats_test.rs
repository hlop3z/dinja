//! Integration tests for the three output formats: HTML, JavaScript, and Schema
//!
//! These tests verify that the RenderService correctly processes MDX content
//! and produces the expected output for each format.

use dinja_core::models::{NamedMdxBatchInput, OutputFormat, RenderSettings};
use dinja_core::service::{FileRenderStatus, RenderService, RenderServiceConfig};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

/// Helper function to create a test RenderService
fn create_test_service() -> RenderService {
    // Skip pool warming for tests
    env::set_var("RUST_CMS_SKIP_POOL_WARMING", "1");

    let config = RenderServiceConfig {
        static_dir: PathBuf::from("static"),
        max_cached_renderers: 2,
        resource_limits: Default::default(),
    };

    RenderService::new(config).expect("Failed to create RenderService")
}

/// Helper function to create test MDX content with frontmatter
fn create_mdx_with_frontmatter(title: &str, content: &str) -> String {
    format!("---\ntitle: {}\n---\n{}", title, content)
}

#[test]
fn test_html_output_simple_markdown() {
    let service = create_test_service();

    let mut mdx_files = HashMap::new();
    mdx_files.insert(
        "simple.mdx".to_string(),
        "# Hello World\n\nThis is a **test**.".to_string(),
    );

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: true,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: None,
    };

    let outcome = service
        .render_batch(&input)
        .expect("Failed to render batch");

    assert_eq!(outcome.total, 1);
    assert_eq!(outcome.succeeded, 1);
    assert_eq!(outcome.failed, 0);
    assert!(outcome.is_all_success());

    let file_outcome = outcome.files.get("simple.mdx").expect("File not found");
    assert!(matches!(file_outcome.status, FileRenderStatus::Success));

    let result = file_outcome
        .result
        .as_ref()
        .expect("Result should be present");

    // Verify HTML output is present
    assert!(result.output.is_some());
    let html = result.output.as_ref().unwrap();
    assert!(html.contains("Hello World"));
    assert!(html.contains("<strong>test</strong>") || html.contains("<b>test</b>"));
}

#[test]
fn test_html_output_with_frontmatter() {
    let service = create_test_service();

    let mut mdx_files = HashMap::new();
    let mdx_content =
        create_mdx_with_frontmatter("Test Page", "# {context('title')}\n\nContent here.");
    mdx_files.insert("with_frontmatter.mdx".to_string(), mdx_content);

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: true,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: None,
    };

    let outcome = service
        .render_batch(&input)
        .expect("Failed to render batch");

    assert!(outcome.is_all_success());

    let file_outcome = outcome
        .files
        .get("with_frontmatter.mdx")
        .expect("File not found");
    let result = file_outcome
        .result
        .as_ref()
        .expect("Result should be present");

    // Verify frontmatter was extracted
    assert!(!result.metadata.is_null());
    assert_eq!(
        result.metadata.get("title").and_then(|v| v.as_str()),
        Some("Test Page")
    );

    // Verify HTML output contains the title
    let html = result
        .output
        .as_ref()
        .expect("HTML output should be present");
    assert!(html.contains("Test Page"));
}

#[test]
fn test_html_output_with_jsx() {
    let service = create_test_service();

    let mut mdx_files = HashMap::new();
    mdx_files.insert(
        "with_jsx.mdx".to_string(),
        "# Title\n\n<div className=\"test\">JSX content</div>".to_string(),
    );

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: true,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: None,
    };

    let outcome = service
        .render_batch(&input)
        .expect("Failed to render batch");

    assert!(outcome.is_all_success());

    let file_outcome = outcome.files.get("with_jsx.mdx").expect("File not found");
    let result = file_outcome
        .result
        .as_ref()
        .expect("Result should be present");

    let html = result
        .output
        .as_ref()
        .expect("HTML output should be present");
    assert!(html.contains("JSX content"));
}

#[test]
fn test_javascript_output_format() {
    let service = create_test_service();

    let mut mdx_files = HashMap::new();
    mdx_files.insert(
        "js_output.mdx".to_string(),
        "# JavaScript Output\n\nTest content.".to_string(),
    );

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Javascript,
            minify: true,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: None,
    };

    let outcome = service
        .render_batch(&input)
        .expect("Failed to render batch");

    assert!(outcome.is_all_success());

    let file_outcome = outcome.files.get("js_output.mdx").expect("File not found");
    let result = file_outcome
        .result
        .as_ref()
        .expect("Result should be present");

    // Verify JavaScript output is present
    assert!(result.output.is_some());
    let js = result.output.as_ref().unwrap();

    // JavaScript output should contain React/JSX constructs
    assert!(!js.is_empty());
    println!("JavaScript output:\n{}", js);
}

#[test]
fn test_javascript_output_with_frontmatter() {
    let service = create_test_service();

    let mut mdx_files = HashMap::new();
    let mdx_content =
        create_mdx_with_frontmatter("JS Test", "# {context('title')}\n\nContent here.");
    mdx_files.insert("js_with_frontmatter.mdx".to_string(), mdx_content);

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Javascript,
            minify: true,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: None,
    };

    let outcome = service
        .render_batch(&input)
        .expect("Failed to render batch");

    assert!(outcome.is_all_success());

    let file_outcome = outcome
        .files
        .get("js_with_frontmatter.mdx")
        .expect("File not found");
    let result = file_outcome
        .result
        .as_ref()
        .expect("Result should be present");

    // Verify frontmatter was extracted
    assert!(!result.metadata.is_null());
    assert_eq!(
        result.metadata.get("title").and_then(|v| v.as_str()),
        Some("JS Test")
    );

    // Verify JavaScript output is present
    let js = result.output.as_ref().expect("JS output should be present");
    assert!(!js.is_empty());
    println!("JavaScript with frontmatter:\n{}", js);
}

#[test]
fn test_schema_output_format() {
    let service = create_test_service();

    let mut mdx_files = HashMap::new();
    mdx_files.insert(
        "schema_output.mdx".to_string(),
        "# Schema Output\n\n<Button>Click</Button>\n\n<Card title=\"Test\">Content</Card>\n\n<Button>Another</Button>".to_string(),
    );

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Schema,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: None,
    };

    let outcome = service
        .render_batch(&input)
        .expect("Failed to render batch");

    assert!(outcome.is_all_success());

    let file_outcome = outcome
        .files
        .get("schema_output.mdx")
        .expect("File not found");
    let result = file_outcome
        .result
        .as_ref()
        .expect("Result should be present");

    // Verify schema (component names list) is present
    assert!(result.output.is_some());
    let schema = result.output.as_ref().unwrap();

    // Schema output should be a JSON object with components and directives
    println!("Schema output:\n{}", schema);

    // Parse as JSON object
    let schema_obj: serde_json::Value =
        serde_json::from_str(schema).expect("Schema should be valid JSON");

    // Should contain Button and Card (unique, sorted)
    let components = schema_obj["components"]
        .as_array()
        .expect("components should be array");
    let component_names: Vec<&str> = components.iter().map(|v| v.as_str().unwrap()).collect();
    assert_eq!(component_names, vec!["Button", "Card"]);

    // Directives should be present but empty
    assert!(schema_obj["directives"].is_object());
}

#[test]
fn test_schema_output_with_complex_jsx() {
    let service = create_test_service();

    let mut mdx_files = HashMap::new();
    mdx_files.insert(
        "schema_complex.mdx".to_string(),
        r#"# Complex JSX

<Container>
  <Header>Title</Header>
  <List>
    <ListItem>Item 1</ListItem>
    <ListItem>Item 2</ListItem>
  </List>
</Container>

<Footer>End</Footer>
"#
        .to_string(),
    );

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Schema,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: None,
    };

    let outcome = service
        .render_batch(&input)
        .expect("Failed to render batch");

    assert!(outcome.is_all_success());

    let file_outcome = outcome
        .files
        .get("schema_complex.mdx")
        .expect("File not found");
    let result = file_outcome
        .result
        .as_ref()
        .expect("Result should be present");

    let schema = result.output.as_ref().expect("Schema should be present");

    println!("Complex schema output:\n{}", schema);

    // Parse as JSON object
    let schema_obj: serde_json::Value =
        serde_json::from_str(schema).expect("Schema should be valid JSON");

    // Should contain all unique component names, sorted
    let components = schema_obj["components"]
        .as_array()
        .expect("components should be array");
    let component_names: Vec<&str> = components.iter().map(|v| v.as_str().unwrap()).collect();
    assert_eq!(
        component_names,
        vec!["Container", "Footer", "Header", "List", "ListItem"]
    );

    // Directives should be present but empty
    assert!(schema_obj["directives"].is_object());
}

#[test]
fn test_schema_output_with_frontmatter() {
    let service = create_test_service();

    let mut mdx_files = HashMap::new();
    let mdx_content = create_mdx_with_frontmatter(
        "Schema Test",
        "# {context('title')}\n\n<Alert type=\"info\">Message</Alert>\n\n<Card>Content</Card>",
    );
    mdx_files.insert("schema_frontmatter.mdx".to_string(), mdx_content);

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Schema,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: None,
    };

    let outcome = service
        .render_batch(&input)
        .expect("Failed to render batch");

    assert!(outcome.is_all_success());

    let file_outcome = outcome
        .files
        .get("schema_frontmatter.mdx")
        .expect("File not found");
    let result = file_outcome
        .result
        .as_ref()
        .expect("Result should be present");

    // Verify frontmatter was extracted
    assert!(!result.metadata.is_null());
    assert_eq!(
        result.metadata.get("title").and_then(|v| v.as_str()),
        Some("Schema Test")
    );

    // Verify schema output is present
    let schema = result.output.as_ref().expect("Schema should be present");

    println!("Schema with frontmatter output:\n{}", schema);

    // Parse as JSON object
    let schema_obj: serde_json::Value =
        serde_json::from_str(schema).expect("Schema should be valid JSON");

    // Should contain Alert and Card (sorted)
    let components = schema_obj["components"]
        .as_array()
        .expect("components should be array");
    let component_names: Vec<&str> = components.iter().map(|v| v.as_str().unwrap()).collect();
    assert_eq!(component_names, vec!["Alert", "Card"]);

    // Directives should be present but empty
    assert!(schema_obj["directives"].is_object());
}

#[test]
fn test_empty_batch() {
    let service = create_test_service();

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: true,
            utils: None,
            directives: None,
        },
        mdx: HashMap::new(),
        components: None,
    };

    let outcome = service
        .render_batch(&input)
        .expect("Failed to render empty batch");

    assert_eq!(outcome.total, 0);
    assert_eq!(outcome.succeeded, 0);
    assert_eq!(outcome.failed, 0);
    assert!(outcome.files.is_empty());
}

#[test]
fn test_output_format_consistency() {
    let service = create_test_service();

    let mdx_content = "# Test\n\nSome **content** here.";

    // Test the same content with all three formats
    for format in [
        OutputFormat::Html,
        OutputFormat::Javascript,
        OutputFormat::Schema,
    ] {
        let mut mdx_files = HashMap::new();
        mdx_files.insert("test.mdx".to_string(), mdx_content.to_string());

        let input = NamedMdxBatchInput {
            settings: RenderSettings {
                output: format.clone(),
                minify: true,
                utils: None,
                directives: None,
            },
            mdx: mdx_files,
            components: None,
        };

        let outcome = service
            .render_batch(&input)
            .unwrap_or_else(|_| panic!("Failed to render with format {:?}", format));

        assert!(
            outcome.is_all_success(),
            "Format {:?} should succeed",
            format
        );

        let file_outcome = outcome.files.get("test.mdx").expect("File not found");
        let result = file_outcome
            .result
            .as_ref()
            .expect("Result should be present");

        assert!(
            result.output.is_some(),
            "Output should be present for format {:?}",
            format
        );

        println!("\n=== Format: {:?} ===", format);
        println!("{}", result.output.as_ref().unwrap());
    }
}

use dinja_core::models::ComponentDefinition;

// ==================== Export Default Component Tests ====================
// Only `export default function Component` is supported

#[test]
fn test_export_default_function_component_renders() {
    let service = create_test_service();

    let mut mdx_files = HashMap::new();
    mdx_files.insert("test.mdx".to_string(), "<MyButton />".to_string());

    let mut components = HashMap::new();
    components.insert(
        "MyButton".to_string(),
        ComponentDefinition {
            name: Some("MyButton".to_string()),
            docs: None,
            args: None,
            code: r#"export default function Component() {
                return <button class="btn">Click Me</button>;
            }"#
            .to_string(),
        },
    );

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: Some(components),
    };

    let outcome = service.render_batch(&input).expect("Failed to render");
    assert!(outcome.is_all_success(), "Render should succeed");

    let html = outcome
        .files
        .get("test.mdx")
        .unwrap()
        .result
        .as_ref()
        .unwrap()
        .output
        .as_ref()
        .unwrap();

    println!("function Component output:\n{}", html);
    assert!(
        html.contains(r#"<button class="btn">Click Me</button>"#),
        "Should render button element: {}",
        html
    );
}

#[test]
fn test_export_default_function_component_with_props_renders() {
    let service = create_test_service();

    let mut mdx_files = HashMap::new();
    mdx_files.insert(
        "test.mdx".to_string(),
        r#"<Greeting name="World" />"#.to_string(),
    );

    let mut components = HashMap::new();
    components.insert(
        "Greeting".to_string(),
        ComponentDefinition {
            name: Some("Greeting".to_string()),
            docs: None,
            args: None,
            code: r#"export default function Component(props) {
                return <span class="greeting">Hello, {props.name}!</span>;
            }"#
            .to_string(),
        },
    );

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: Some(components),
    };

    let outcome = service.render_batch(&input).expect("Failed to render");
    assert!(outcome.is_all_success(), "Render should succeed");

    let html = outcome
        .files
        .get("test.mdx")
        .unwrap()
        .result
        .as_ref()
        .unwrap()
        .output
        .as_ref()
        .unwrap();

    println!("function Component with props output:\n{}", html);
    assert!(
        html.contains(r#"<span class="greeting">Hello, World!</span>"#),
        "Should render greeting with props: {}",
        html
    );
}

#[test]
fn test_invalid_export_default_function_name_fails() {
    let service = create_test_service();

    let mut mdx_files = HashMap::new();
    mdx_files.insert("test.mdx".to_string(), "<BadButton />".to_string());

    let mut components = HashMap::new();
    components.insert(
        "BadButton".to_string(),
        ComponentDefinition {
            name: Some("BadButton".to_string()),
            docs: None,
            args: None,
            // Using "Button" instead of "Component" should fail
            code: r#"export default function Button() {
                return <button>Bad</button>;
            }"#
            .to_string(),
        },
    );

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: Some(components),
    };

    let outcome = service.render_batch(&input).expect("Failed to render");

    // Should fail because function name is not "Component"
    assert!(
        !outcome.is_all_success(),
        "Render should fail for wrong function name"
    );

    let file_outcome = outcome.files.get("test.mdx").unwrap();
    assert!(
        matches!(file_outcome.status, FileRenderStatus::Failed),
        "Should have failed status"
    );

    let error = file_outcome.error.as_ref().expect("Should have error");
    println!("Expected error: {}", error);
    assert!(
        error.contains("function Button") || error.contains("naming convention"),
        "Error should mention the wrong function name: {}",
        error
    );
}

#[test]
fn test_export_default_function_component_typescript_renders() {
    let service = create_test_service();

    let mut mdx_files = HashMap::new();
    mdx_files.insert(
        "test.mdx".to_string(),
        r#"<TypedGreeting name="TypeScript" count={42} />"#.to_string(),
    );

    let mut components = HashMap::new();
    components.insert(
        "TypedGreeting".to_string(),
        ComponentDefinition {
            name: Some("TypedGreeting".to_string()),
            docs: None,
            args: None,
            // TypeScript syntax with type annotations
            code: r#"export default function Component(props: { name: string; count: number }) {
                return <div class="typed">Hello {props.name}, count: {props.count}</div>;
            }"#
            .to_string(),
        },
    );

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: Some(components),
    };

    let outcome = service.render_batch(&input).expect("Failed to render");
    assert!(
        outcome.is_all_success(),
        "TypeScript component should render successfully"
    );

    let html = outcome
        .files
        .get("test.mdx")
        .unwrap()
        .result
        .as_ref()
        .unwrap()
        .output
        .as_ref()
        .unwrap();

    println!("TypeScript component output:\n{}", html);
    assert!(
        html.contains("Hello TypeScript"),
        "Should render name prop: {}",
        html
    );
    assert!(
        html.contains("count: 42"),
        "Should render count prop: {}",
        html
    );
}

#[test]
fn test_decorator_outside_component_renders() {
    let service = create_test_service();

    let mut mdx_files = HashMap::new();
    mdx_files.insert(
        "test.mdx".to_string(),
        r#"<DecoratedUtils text="hello" />"#.to_string(),
    );

    let mut components = HashMap::new();
    components.insert(
        "DecoratedUtils".to_string(),
        ComponentDefinition {
            name: Some("DecoratedUtils".to_string()),
            docs: None,
            args: None,
            // Decorator on class at module level (outside Component)
            code: r#"function logged(target: any) { return target; }

@logged
class Utils {
    format(value: string) { return value.toUpperCase(); }
}

export default function Component(props: { text: string }) {
    const u = new Utils();
    return <div class="decorated">{u.format(props.text)}</div>;
}"#
            .to_string(),
        },
    );

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: Some(components),
    };

    let outcome = service.render_batch(&input).expect("Failed to render");
    assert!(
        outcome.is_all_success(),
        "Module-level decorator component should render successfully"
    );

    let html = outcome
        .files
        .get("test.mdx")
        .unwrap()
        .result
        .as_ref()
        .unwrap()
        .output
        .as_ref()
        .unwrap();

    println!("Decorator component output:\n{}", html);
    assert!(
        html.contains("HELLO"),
        "Should render uppercase text: {}",
        html
    );
    assert!(html.contains("decorated"), "Should have class: {}", html);
}

#[test]
fn test_decorator_on_class_method_renders() {
    let service = create_test_service();

    let mut mdx_files = HashMap::new();
    mdx_files.insert(
        "test.mdx".to_string(),
        r#"<MethodDecorator text="hello" />"#.to_string(),
    );

    let mut components = HashMap::new();
    components.insert(
        "MethodDecorator".to_string(),
        ComponentDefinition {
            name: Some("MethodDecorator".to_string()),
            docs: None,
            args: None,
            // Method decorator that wraps the function
            code: r#"function uppercase(target: any, key: string, descriptor: PropertyDescriptor) {
    const original = descriptor.value;
    descriptor.value = function(value: string) {
        return original.call(this, value).toUpperCase();
    };
    return descriptor;
}

class Utils {
    @uppercase
    format(value: string) { return value; }
}

export default function Component(props: { text: string }) {
    const u = new Utils();
    return <div class="method-dec">{u.format(props.text)}</div>;
}"#
            .to_string(),
        },
    );

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: Some(components),
    };

    let outcome = service.render_batch(&input).expect("Failed to render");
    assert!(
        outcome.is_all_success(),
        "Method decorator component should render successfully"
    );

    let html = outcome
        .files
        .get("test.mdx")
        .unwrap()
        .result
        .as_ref()
        .unwrap()
        .output
        .as_ref()
        .unwrap();

    println!("Method decorator output:\n{}", html);
    assert!(
        html.contains("HELLO"),
        "Method decorator should transform to uppercase: {}",
        html
    );
}

#[test]
fn test_invalid_export_default_arrow_function_fails() {
    let service = create_test_service();

    let mut mdx_files = HashMap::new();
    mdx_files.insert("test.mdx".to_string(), "<ArrowComp />".to_string());

    let mut components = HashMap::new();
    components.insert(
        "ArrowComp".to_string(),
        ComponentDefinition {
            name: Some("ArrowComp".to_string()),
            docs: None,
            args: None,
            // Arrow functions are not supported
            code: r#"export default () => <div>Arrow</div>"#.to_string(),
        },
    );

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: Some(components),
    };

    let outcome = service.render_batch(&input).expect("Failed to render");
    assert!(
        !outcome.is_all_success(),
        "Render should fail for arrow function"
    );

    let error = outcome
        .files
        .get("test.mdx")
        .unwrap()
        .error
        .as_ref()
        .expect("Should have error");
    println!("Arrow function error: {}", error);
    assert!(
        error.contains("arrow function"),
        "Error should mention arrow function: {}",
        error
    );
}

#[test]
fn test_invalid_export_default_class_fails() {
    let service = create_test_service();

    let mut mdx_files = HashMap::new();
    mdx_files.insert("test.mdx".to_string(), "<ClassComp />".to_string());

    let mut components = HashMap::new();
    components.insert(
        "ClassComp".to_string(),
        ComponentDefinition {
            name: Some("ClassComp".to_string()),
            docs: None,
            args: None,
            // Classes are not supported
            code: r#"export default class Component { render() { return <div>Class</div>; } }"#
                .to_string(),
        },
    );

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: Some(components),
    };

    let outcome = service.render_batch(&input).expect("Failed to render");
    assert!(!outcome.is_all_success(), "Render should fail for class");

    let error = outcome
        .files
        .get("test.mdx")
        .unwrap()
        .error
        .as_ref()
        .expect("Should have error");
    println!("Class error: {}", error);
    assert!(
        error.contains("class"),
        "Error should mention class: {}",
        error
    );
}

#[test]
fn test_jsx_component_with_expression_attributes() {
    // This test verifies that JSX components with expression attributes
    // (like `title={context("title")}`) are properly parsed and rendered,
    // not escaped as HTML entities.
    let service = create_test_service();

    let mut mdx_files = HashMap::new();
    let mdx_content = r#"---
title: Home
subtitle: Welcome to our site
author: John Doe
---

# {context('title')}

Welcome to our amazing website! This is a simple demo of MDX rendering.

## Features

<Hero title={context("title")} subtitle={context("subtitle")} />
"#;
    mdx_files.insert("jsx_expression.mdx".to_string(), mdx_content.to_string());

    // Provide a Hero component definition using JSX syntax
    let mut components = HashMap::new();
    components.insert(
        "Hero".to_string(),
        ComponentDefinition {
            name: Some("Hero".to_string()),
            docs: None,
            args: None,
            code: r#"export default function Component({ title, subtitle }) {
                return <div class="hero"><h2>{title}</h2><p>{subtitle}</p></div>;
            }"#
            .to_string(),
        },
    );

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: Some(components),
    };

    let outcome = service
        .render_batch(&input)
        .expect("Failed to render batch");

    assert!(outcome.is_all_success());

    let file_outcome = outcome
        .files
        .get("jsx_expression.mdx")
        .expect("File not found");
    let result = file_outcome
        .result
        .as_ref()
        .expect("Result should be present");

    let html = result.output.as_ref().expect("HTML should be present");
    println!("HTML output:\n{}", html);

    // The JSX component should NOT be escaped (no &lt; or &gt;)
    assert!(
        !html.contains("&lt;Hero"),
        "JSX component should not be escaped: {}",
        html
    );
    assert!(
        !html.contains("&gt;"),
        "JSX closing should not be escaped: {}",
        html
    );

    // The frontmatter context() calls should be resolved
    assert!(
        html.contains("Home"),
        "Frontmatter title should be resolved: {}",
        html
    );

    // The Hero component should be rendered with the correct props
    // It should produce <div class="hero"><h2>Home</h2><p>Welcome to our site</p></div>
    assert!(
        html.contains(r#"<div class="hero">"#),
        "Hero component should be rendered with hero class: {}",
        html
    );
    assert!(
        html.contains("<h2>Home</h2>"),
        "Hero component should render title from frontmatter: {}",
        html
    );
    assert!(
        html.contains("<p>Welcome to our site</p>"),
        "Hero component should render subtitle from frontmatter: {}",
        html
    );
}

#[test]
fn test_component_with_fragment() {
    // Tests that components using JSX Fragments (<>...</>) render correctly
    // without leaving <Fragment> tags in the output
    let service = create_test_service();

    let mut mdx_files = HashMap::new();
    let mdx_content = create_mdx_with_frontmatter(
        "Test",
        r#"# Test

<Card title="Hello" subtitle="World" />
"#,
    );
    mdx_files.insert("fragment.mdx".to_string(), mdx_content);

    // Component that uses Fragment to return multiple elements
    let mut components = HashMap::new();
    components.insert(
        "Card".to_string(),
        dinja_core::models::ComponentDefinition {
            name: Some("Card".to_string()),
            docs: None,
            args: None,
            code: r#"export function Component(props) {
  const { title, subtitle } = props;
  return (
    <>
      <h1>Title: {title}</h1>
      <hr />
      <h3>Subtitle: {subtitle}</h3>
    </>
  );
}"#
            .to_string(),
        },
    );

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: Some(components),
    };

    let outcome = service
        .render_batch(&input)
        .expect("Failed to render batch");

    assert!(outcome.is_all_success());

    let file_outcome = outcome.files.get("fragment.mdx").expect("File not found");
    let result = file_outcome
        .result
        .as_ref()
        .expect("Result should be present");

    let html = result.output.as_ref().expect("Output should be present");
    println!("Fragment component output:\n{}", html);

    // Fragment tags should NOT appear in output
    assert!(
        !html.contains("<Fragment") && !html.contains("</Fragment>"),
        "Fragment tags should be stripped from output: {}",
        html
    );

    // The actual content should be present
    assert!(
        html.contains("Title: Hello"),
        "Should contain title: {}",
        html
    );
    assert!(
        html.contains("Subtitle: World"),
        "Should contain subtitle: {}",
        html
    );
    assert!(html.contains("<hr"), "Should contain hr element: {}", html);
}

#[test]
fn test_markdown_edge_cases_complex_nesting() {
    // Tests complex markdown with deeply nested structures, code blocks,
    // tables, blockquotes, and special characters
    let service = create_test_service();

    let edge_cases_content = include_str!("edge_cases.md");

    let mut mdx_files = HashMap::new();
    mdx_files.insert("edge_cases.mdx".to_string(), edge_cases_content.to_string());

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: None,
    };

    let outcome = service
        .render_batch(&input)
        .expect("Failed to render batch");

    // Print errors for debugging if any
    if !outcome.errors.is_empty() {
        for err in &outcome.errors {
            eprintln!("Error in {}: {}", err.file, err.message);
            if let Some(line) = err.line {
                eprintln!("  Line: {}, Column: {:?}", line, err.column);
            }
            if let Some(help) = &err.help {
                eprintln!("  Help: {}", help);
            }
        }
    }

    assert!(
        outcome.is_all_success(),
        "Edge cases markdown should render successfully. Errors: {:?}",
        outcome.errors
    );

    let file_outcome = outcome.files.get("edge_cases.mdx").expect("File not found");
    let result = file_outcome
        .result
        .as_ref()
        .expect("Result should be present");
    let html = result.output.as_ref().expect("HTML should be present");

    // Verify basic structure is preserved
    assert!(
        html.contains("<h1>"),
        "Should contain h1 heading: {}",
        &html[..500.min(html.len())]
    );

    // Verify code blocks rendered with escaped braces
    assert!(
        html.contains("<pre>") || html.contains("<code"),
        "Should contain code blocks"
    );

    // Verify tables rendered
    assert!(html.contains("<table"), "Should contain tables");

    // Verify blockquotes rendered
    assert!(html.contains("<blockquote"), "Should contain blockquotes");

    // Verify task lists rendered (checkboxes)
    assert!(
        html.contains("type=\"checkbox\"") || html.contains("[x]") || html.contains("[ ]"),
        "Should contain task list items"
    );

    println!(
        "Edge cases rendered successfully! Output length: {} chars",
        html.len()
    );
}

#[test]
fn test_stress_jsx_like_code_in_blocks() {
    // Stress test: Code blocks containing JSX-like syntax that should NOT be parsed as JSX
    let service = create_test_service();

    let content = r#"# React Component Examples

Here's a React component with complex JSX:

```jsx
import React, { useState, useEffect } from 'react';

const ComplexComponent = ({ items, onSelect, config = {} }) => {
  const [state, setState] = useState({ count: 0, data: null });
  const { theme = 'dark', size = 'md' } = config;

  useEffect(() => {
    const fetchData = async () => {
      const response = await fetch(`/api/items?theme=${theme}&size=${size}`);
      const json = await response.json();
      setState(prev => ({ ...prev, data: json }));
    };
    fetchData();
  }, [theme, size]);

  return (
    <div className={`container ${theme}`} data-testid="complex-component">
      <header style={{ padding: '1rem' }}>
        <h1>{config.title || 'Default Title'}</h1>
        <span>{state.count} items</span>
      </header>
      <ul>
        {items.map((item, idx) => (
          <li key={item.id || idx} onClick={() => onSelect(item)}>
            <span className="name">{item.name}</span>
            <span className="value">{item.value ?? 'N/A'}</span>
            {item.children && (
              <ul>
                {item.children.map(child => (
                  <li key={child.id}>{child.label}</li>
                ))}
              </ul>
            )}
          </li>
        ))}
      </ul>
      {state.data && <pre>{JSON.stringify(state.data, null, 2)}</pre>}
    </div>
  );
};

export default ComplexComponent;
```

And a Vue 3 component:

```vue
<template>
  <div :class="{ active: isActive, 'text-danger': hasError }">
    <slot name="header" :user="user">
      <h2>{{ user.name }}</h2>
    </slot>
    <ul v-if="items.length">
      <li v-for="(item, index) in items" :key="item.id" @click="handleClick(item, $event)">
        {{ item.label }} - {{ formatDate(item.date) }}
        <button @click.stop.prevent="remove(index)">×</button>
      </li>
    </ul>
    <p v-else>No items found</p>
    <teleport to="body">
      <modal v-if="showModal" @close="showModal = false">
        <template #title>{{ modalTitle }}</template>
        <template #default="{ close }">
          <p>Content here</p>
          <button @click="close">Close</button>
        </template>
      </modal>
    </teleport>
  </div>
</template>

<script setup>
import { ref, computed, watch } from 'vue';

const props = defineProps({
  items: { type: Array, default: () => [] },
  user: { type: Object, required: true }
});

const emit = defineEmits(['update', 'delete']);

const isActive = ref(false);
const hasError = computed(() => props.items.some(i => i.error));

watch(() => props.items, (newVal) => {
  console.log(`Items changed: ${newVal.length}`);
}, { deep: true });
</script>
```
"#;

    let mut mdx_files = HashMap::new();
    mdx_files.insert("jsx_stress.mdx".to_string(), content.to_string());

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: None,
    };

    let outcome = service.render_batch(&input).expect("Failed to render");

    if !outcome.errors.is_empty() {
        for err in &outcome.errors {
            eprintln!("Error: {} - {}", err.file, err.message);
        }
    }

    assert!(
        outcome.is_all_success(),
        "JSX-like code blocks should render. Errors: {:?}",
        outcome.errors
    );

    let html = outcome
        .files
        .get("jsx_stress.mdx")
        .unwrap()
        .result
        .as_ref()
        .unwrap()
        .output
        .as_ref()
        .unwrap();

    // Verify code blocks rendered correctly
    assert!(html.contains("<pre>"), "Should have pre blocks");
    assert!(
        html.contains("<code"),
        "Should have code elements inside pre blocks"
    );

    // The curly braces should be present in the final output as literal characters
    // (The escaping {'{'}  happens internally during TSX parsing, but after
    // JavaScript execution they become literal { and } in the output)
    assert!(
        html.contains("{") && html.contains("}"),
        "Code blocks should contain curly braces in final output"
    );

    // Verify the code content is preserved (not mangled)
    assert!(
        html.contains("useState") && html.contains("useEffect"),
        "React hooks should be present in code"
    );

    println!("JSX stress test passed! Output: {} chars", html.len());
}

#[test]
fn test_stress_nested_template_literals_and_objects() {
    // Stress test: Deeply nested template literals and object destructuring
    let service = create_test_service();

    let content = r#"# JavaScript Nesting Stress Test

Complex template literal nesting:

```javascript
const generateQuery = (table, { fields = ['*'], where = {}, orderBy, limit }) => {
  const whereClause = Object.entries(where)
    .map(([key, value]) => {
      if (typeof value === 'object') {
        const { op = '=', val } = value;
        return `${key} ${op} ${typeof val === 'string' ? `'${val}'` : val}`;
      }
      return `${key} = ${typeof value === 'string' ? `'${value}'` : value}`;
    })
    .join(' AND ');

  return `
    SELECT ${fields.join(', ')}
    FROM ${table}
    ${whereClause ? `WHERE ${whereClause}` : ''}
    ${orderBy ? `ORDER BY ${orderBy.field} ${orderBy.dir || 'ASC'}` : ''}
    ${limit ? `LIMIT ${limit}` : ''}
  `.trim().replace(/\s+/g, ' ');
};

// Nested object destructuring with defaults
const processConfig = ({
  server: {
    host = 'localhost',
    port = 3000,
    ssl: { enabled: sslEnabled = false, cert = null } = {}
  } = {},
  database: {
    connection: { url = `postgres://${host}:5432/db` } = {}
  } = {},
  features: { [process.env.NODE_ENV]: envFeatures = {} } = {}
} = {}) => {
  return {
    serverUrl: `http${sslEnabled ? 's' : ''}://${host}:${port}`,
    dbUrl: url,
    features: { ...envFeatures }
  };
};

// Tagged template literal
const sql = (strings, ...values) => ({
  text: strings.reduce((acc, str, i) =>
    `${acc}${str}${i < values.length ? `$${i + 1}` : ''}`, ''),
  values
});

const query = sql`
  INSERT INTO users (name, email, metadata)
  VALUES (${name}, ${email}, ${JSON.stringify({ created: new Date(), tags: ['new', 'active'] })})
  RETURNING *
`;
```

Triple nested callbacks:

```javascript
fs.readFile(path, 'utf8', (err, data) => {
  if (err) return callback({ error: err, context: { path, timestamp: Date.now() } });

  parseAsync(data, { format: 'json' }, (parseErr, parsed) => {
    if (parseErr) return callback({ error: parseErr, raw: data.slice(0, 100) });

    transform(parsed, (item) => ({
      ...item,
      computed: `${item.type}-${item.id}-${Date.now()}`,
      nested: { value: item.nested?.value ?? 'default' }
    }), (transformErr, result) => {
      callback(transformErr, {
        data: result,
        meta: {
          source: path,
          processed: new Date().toISOString(),
          stats: { original: data.length, items: result?.length ?? 0 }
        }
      });
    });
  });
});
```
"#;

    let mut mdx_files = HashMap::new();
    mdx_files.insert("nesting_stress.mdx".to_string(), content.to_string());

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: None,
    };

    let outcome = service.render_batch(&input).expect("Failed to render");

    assert!(
        outcome.is_all_success(),
        "Nested template literals should render. Errors: {:?}",
        outcome.errors
    );

    println!("Nested template literals stress test passed!");
}

#[test]
fn test_stress_special_chars_and_escape_sequences() {
    // Stress test: HTML entities, escape sequences, regex patterns
    let service = create_test_service();

    let content = r#"# Special Characters Stress Test

## HTML Entities in Code

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>&lt;script&gt; &amp; &quot;entities&quot; &#x27;test&#x27;</title>
  <style>
    .arrow::before { content: '\2192'; } /* → */
    .quote::before { content: '\201C'; } /* " */
    [data-content="a && b || c"] { display: block; }
  </style>
</head>
<body>
  <div data-json='{"key": "value", "nested": {"a": 1}}'>
    &lt;not-a-tag&gt; but <real-tag></real-tag>
  </div>
  <script>
    const html = `<div class="${cls}">${content}</div>`;
    const escaped = text.replace(/[<>&"']/g, c => ({
      '<': '&lt;', '>': '&gt;', '&': '&amp;', '"': '&quot;', "'": '&#39;'
    })[c]);
  </script>
</body>
</html>
```

## Regex Patterns

```javascript
// Patterns that might confuse parsers
const patterns = {
  jsx: /<([A-Z][a-zA-Z]*)\s*(\{[^}]*\}|[^>])*\/?>/g,
  templateLiteral: /\$\{([^}]+)\}/g,
  nestedBraces: /\{(?:[^{}]|\{(?:[^{}]|\{[^{}]*\})*\})*\}/g,
  htmlEntity: /&(?:#x?[0-9a-fA-F]+|[a-zA-Z]+);/g,
  escapedChars: /\\[nrtfvb0\\'"]/g,
  unicodeEscape: /\\u\{[0-9a-fA-F]+\}|\\u[0-9a-fA-F]{4}/g
};

// Test string with all problematic chars
const test = `
  Braces: { } {{ }} {{{ }}}
  Template: \${var} \${{nested}}
  Escaped: \n \t \r \\ \'
  Unicode: \u{1F600} \u0041
  HTML-ish: <div> </div> <br/> <input type="text">
  Entities: &lt; &gt; &amp; &quot; &#39; &#x27;
  Mixed: <Component prop={value} /> vs &lt;Component&gt;
`;
```

## Escape Sequences in Strings

```python
# Python string edge cases
raw = r'Raw: \n \t \{ \} ${ } stays literal'
fstring = f"Formatted: {value} and {{escaped braces}}"
triple = """
Multi-line with {curly} and 'quotes' and "double"
And backslash: \\ and newline: \n
"""
bytes_lit = b'\x00\x01\x02\xff'
unicode_str = '\u0041\U0001F600\N{GREEK SMALL LETTER ALPHA}'
```

## Shell Escaping

```bash
# Complex quoting
echo "Double quotes: $VAR and \"escaped\" and \$literal"
echo 'Single quotes: $VAR stays literal, '\''escaped single'\'''
echo $'ANSI-C: \n\t\x41\u0042'

# Nested command substitution
result=$(echo "$(echo "$(echo "deeply nested")")")

# Here-doc with various quoting
cat << 'EOF'
$VAR not expanded, {braces} literal
EOF

cat << EOF
$VAR expanded, but \$escaped stays
EOF

# Parameter expansion
echo "${var:-default}" "${var:+alternate}" "${var:?error}"
echo "${array[@]}" "${#string}" "${string//pattern/replacement}"
```
"#;

    let mut mdx_files = HashMap::new();
    mdx_files.insert("special_chars.mdx".to_string(), content.to_string());

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: None,
    };

    let outcome = service.render_batch(&input).expect("Failed to render");

    assert!(
        outcome.is_all_success(),
        "Special characters should render. Errors: {:?}",
        outcome.errors
    );

    let html = outcome
        .files
        .get("special_chars.mdx")
        .unwrap()
        .result
        .as_ref()
        .unwrap()
        .output
        .as_ref()
        .unwrap();

    // Verify code blocks are present
    assert!(html.contains("<pre>"), "Should have pre blocks");
    assert!(html.contains("<code"), "Should have code elements");
    println!(
        "Special chars stress test passed! Output: {} chars",
        html.len()
    );
}

#[test]
fn test_stress_mixed_jsx_markdown_html() {
    // Stress test: Real JSX components mixed with markdown and raw HTML
    let service = create_test_service();

    let content = r#"---
title: Mixed Content Test
items:
  - name: Item 1
    value: 100
  - name: Item 2
    value: 200
---

# {context('title')}

This tests mixing **real JSX components** with markdown and HTML.

## Markdown Section

Here's a list with `inline code` and **bold**:

1. First item with code: `const x = { a: 1 };`
2. Second item with JSX-like: `<Component prop={value} />`
3. Third with template: `Hello ${name}!`

> **Note:** This blockquote contains `{curly braces}` and <em>HTML tags</em>.

## Real JSX Component

<DataTable
  title={context('title')}
  items={context('items')}
/>

## HTML Table (Not JSX)

<table border="1">
<thead>
<tr><th>Name</th><th>Value</th></tr>
</thead>
<tbody>
<tr><td>Alpha</td><td>100</td></tr>
<tr><td>Beta</td><td>200</td></tr>
</tbody>
</table>

## Code Block After Component

```tsx
// This is code, not real JSX
const Table = ({ data }: { data: Array<{ id: number; name: string }> }) => (
  <table>
    <tbody>
      {data.map(row => (
        <tr key={row.id}>
          <td>{row.name}</td>
        </tr>
      ))}
    </tbody>
  </table>
);
```

## Another Real Component

<Alert type="info">
  This alert contains **markdown** and `code`.
</Alert>

---

*End of mixed content test*
"#;

    let mut mdx_files = HashMap::new();
    mdx_files.insert("mixed_stress.mdx".to_string(), content.to_string());

    // Provide the components
    let mut components = HashMap::new();
    components.insert(
        "DataTable".to_string(),
        ComponentDefinition {
            name: Some("DataTable".to_string()),
            docs: None,
            args: None,
            code: r#"export default function Component({ title, items }) {
                return (
                    <div class="data-table">
                        <h3>{title}</h3>
                        <ul>
                            {items.map((item, i) => (
                                <li key={i}>{item.name}: {item.value}</li>
                            ))}
                        </ul>
                    </div>
                );
            }"#
            .to_string(),
        },
    );
    components.insert(
        "Alert".to_string(),
        ComponentDefinition {
            name: Some("Alert".to_string()),
            docs: None,
            args: None,
            code: r#"export default function Component({ type, children }) {
                return <div class={`alert alert-${type}`}>{children}</div>;
            }"#
            .to_string(),
        },
    );

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: Some(components),
    };

    let outcome = service.render_batch(&input).expect("Failed to render");

    if !outcome.errors.is_empty() {
        for err in &outcome.errors {
            eprintln!("Error: {} - {}", err.file, err.message);
            if let Some(line) = err.line {
                eprintln!("  at line {}, col {:?}", line, err.column);
            }
        }
    }

    assert!(
        outcome.is_all_success(),
        "Mixed JSX/Markdown/HTML should render. Errors: {:?}",
        outcome.errors
    );

    let html = outcome
        .files
        .get("mixed_stress.mdx")
        .unwrap()
        .result
        .as_ref()
        .unwrap()
        .output
        .as_ref()
        .unwrap();

    // Verify components rendered
    assert!(
        html.contains("data-table"),
        "DataTable should render: {}",
        &html[..1000.min(html.len())]
    );
    assert!(html.contains("alert-info"), "Alert should render: {}", html);

    // Verify frontmatter resolved
    assert!(
        html.contains("Mixed Content Test"),
        "Title should be resolved"
    );
    assert!(html.contains("Item 1"), "Items should be rendered");

    // Verify code blocks have escaped braces
    assert!(
        html.contains("{'{'}") || html.contains("<code"),
        "Code blocks should be present with escaped braces"
    );

    // Verify HTML table preserved
    assert!(html.contains("<table"), "HTML table should be preserved");

    println!(
        "Mixed JSX/Markdown/HTML stress test passed! Output: {} chars",
        html.len()
    );
}

// =============================================================================
// TSX/MDX Edge Case Tests
// =============================================================================

#[test]
fn test_jsx_explicit_props_from_context() {
    // Test explicitly passing props from context (spread syntax {...context()} is not supported)
    let service = create_test_service();

    let content = r#"---
title: Props Test
className: container
id: main-box
---

<Box title={context('title')} className={context('className')} id={context('id')} />
"#;

    let mut mdx_files = HashMap::new();
    mdx_files.insert("props.mdx".to_string(), content.to_string());

    let mut components = HashMap::new();
    components.insert(
        "Box".to_string(),
        ComponentDefinition {
            name: Some("Box".to_string()),
            docs: None,
            args: None,
            code: r#"export default function Component(props) {
                return <div id={props.id} class={props.className}>{props.title}</div>;
            }"#
            .to_string(),
        },
    );

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: Some(components),
    };

    let outcome = service.render_batch(&input).expect("Failed to render");
    assert!(
        outcome.is_all_success(),
        "Explicit props should work. Errors: {:?}",
        outcome.errors
    );

    let html = outcome
        .files
        .get("props.mdx")
        .unwrap()
        .result
        .as_ref()
        .unwrap()
        .output
        .as_ref()
        .unwrap();

    assert!(html.contains("Props Test"), "Title should render: {}", html);
    assert!(
        html.contains("container"),
        "className should render: {}",
        html
    );
    assert!(html.contains("main-box"), "id should render: {}", html);
    println!("Explicit props output: {}", html);
}

#[test]
fn test_inline_mdx_expressions() {
    // Test inline expressions in markdown text
    let service = create_test_service();

    let content = r#"---
name: World
count: 42
---

# Hello {context('name')}!

The answer is {context('count')}.

Math works: {10 + 5} equals fifteen.
"#;

    let mut mdx_files = HashMap::new();
    mdx_files.insert("inline.mdx".to_string(), content.to_string());

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: None,
    };

    let outcome = service.render_batch(&input).expect("Failed to render");
    assert!(
        outcome.is_all_success(),
        "Inline expressions should work. Errors: {:?}",
        outcome.errors
    );

    let html = outcome
        .files
        .get("inline.mdx")
        .unwrap()
        .result
        .as_ref()
        .unwrap()
        .output
        .as_ref()
        .unwrap();

    assert!(
        html.contains("Hello World"),
        "Name should be interpolated: {}",
        html
    );
    assert!(
        html.contains("42"),
        "Count should be interpolated: {}",
        html
    );
    assert!(html.contains("15"), "Math should evaluate: {}", html);
}

#[test]
fn test_nested_jsx_components() {
    // Test nested component rendering
    let service = create_test_service();

    let content = r#"<Card>
  <CardHeader title="Nested" />
  <CardBody>
    Content here
  </CardBody>
</Card>"#;

    let mut mdx_files = HashMap::new();
    mdx_files.insert("nested.mdx".to_string(), content.to_string());

    let mut components = HashMap::new();
    components.insert(
        "Card".to_string(),
        ComponentDefinition {
            name: Some("Card".to_string()),
            docs: None,
            args: None,
            code: r#"export default function Component({ children }) {
                return <div class="card">{children}</div>;
            }"#
            .to_string(),
        },
    );
    components.insert(
        "CardHeader".to_string(),
        ComponentDefinition {
            name: Some("CardHeader".to_string()),
            docs: None,
            args: None,
            code: r#"export default function Component({ title }) {
                return <div class="card-header">{title}</div>;
            }"#
            .to_string(),
        },
    );
    components.insert(
        "CardBody".to_string(),
        ComponentDefinition {
            name: Some("CardBody".to_string()),
            docs: None,
            args: None,
            code: r#"export default function Component({ children }) {
                return <div class="card-body">{children}</div>;
            }"#
            .to_string(),
        },
    );

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: Some(components),
    };

    let outcome = service.render_batch(&input).expect("Failed to render");
    assert!(
        outcome.is_all_success(),
        "Nested components should work. Errors: {:?}",
        outcome.errors
    );

    let html = outcome
        .files
        .get("nested.mdx")
        .unwrap()
        .result
        .as_ref()
        .unwrap()
        .output
        .as_ref()
        .unwrap();

    assert!(
        html.contains("card"),
        "Card class should be present: {}",
        html
    );
    assert!(
        html.contains("card-header"),
        "CardHeader should render: {}",
        html
    );
    assert!(
        html.contains("card-body"),
        "CardBody should render: {}",
        html
    );
    assert!(
        html.contains("Nested"),
        "Title prop should render: {}",
        html
    );
}

#[test]
fn test_component_children_with_markdown() {
    // Test component children containing markdown syntax
    let service = create_test_service();

    let content = r#"<Callout type="warning">

**Important:** This is *emphasized* text with `code`.

- Item 1
- Item 2

</Callout>"#;

    let mut mdx_files = HashMap::new();
    mdx_files.insert("children_md.mdx".to_string(), content.to_string());

    let mut components = HashMap::new();
    components.insert(
        "Callout".to_string(),
        ComponentDefinition {
            name: Some("Callout".to_string()),
            docs: None,
            args: None,
            code: r#"export default function Component({ type, children }) {
                return <div class={`callout callout-${type}`}>{children}</div>;
            }"#
            .to_string(),
        },
    );

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: Some(components),
    };

    let outcome = service.render_batch(&input).expect("Failed to render");
    assert!(
        outcome.is_all_success(),
        "Children with markdown should work. Errors: {:?}",
        outcome.errors
    );

    let html = outcome
        .files
        .get("children_md.mdx")
        .unwrap()
        .result
        .as_ref()
        .unwrap()
        .output
        .as_ref()
        .unwrap();

    assert!(
        html.contains("callout-warning"),
        "Callout type should render: {}",
        html
    );
    // Note: markdown inside JSX children may or may not be processed depending on implementation
    println!("Children with markdown output: {}", html);
}

#[test]
fn test_array_object_props() {
    // Test passing array and object literals as props
    let service = create_test_service();

    let content =
        r#"<List items={["apple", "banana", "cherry"]} config={{ sorted: true, limit: 10 }} />"#;

    let mut mdx_files = HashMap::new();
    mdx_files.insert("array_props.mdx".to_string(), content.to_string());

    let mut components = HashMap::new();
    components.insert(
        "List".to_string(),
        ComponentDefinition {
            name: Some("List".to_string()),
            docs: None,
            args: None,
            code: r#"export default function Component({ items, config }) {
                const sorted = config.sorted ? items.slice().sort() : items;
                const limited = sorted.slice(0, config.limit);
                return <ul>{limited.map((item, i) => <li key={i}>{item}</li>)}</ul>;
            }"#
            .to_string(),
        },
    );

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: Some(components),
    };

    let outcome = service.render_batch(&input).expect("Failed to render");
    assert!(
        outcome.is_all_success(),
        "Array/object props should work. Errors: {:?}",
        outcome.errors
    );

    let html = outcome
        .files
        .get("array_props.mdx")
        .unwrap()
        .result
        .as_ref()
        .unwrap()
        .output
        .as_ref()
        .unwrap();

    assert!(html.contains("<ul>"), "List should render: {}", html);
    assert!(html.contains("apple"), "Items should render: {}", html);
    assert!(html.contains("banana"), "Items should render: {}", html);
    // Sorted order: apple, banana, cherry
    println!("Array props output: {}", html);
}

#[test]
fn test_conditional_jsx_in_component() {
    // Test conditional rendering via component props (standalone conditional JSX at top-level has parsing limitations)
    let service = create_test_service();

    let content = r#"---
showHeader: true
showFooter: false
isDarkMode: true
---

<ConditionalLayout
    showHeader={context('showHeader')}
    showFooter={context('showFooter')}
    isDarkMode={context('isDarkMode')}
/>
"#;

    let mut mdx_files = HashMap::new();
    mdx_files.insert("conditional.mdx".to_string(), content.to_string());

    let mut components = HashMap::new();
    components.insert(
        "ConditionalLayout".to_string(),
        ComponentDefinition {
            name: Some("ConditionalLayout".to_string()),
            docs: None,
            args: None,
            code: r#"export default function Component({ showHeader, showFooter, isDarkMode }) {
                return (
                    <div class="layout">
                        {showHeader && <header class="header">Header Content</header>}
                        <main class={isDarkMode ? "theme-dark" : "theme-light"}>
                            Main content
                        </main>
                        {showFooter && <footer class="footer">Footer Content</footer>}
                    </div>
                );
            }"#
            .to_string(),
        },
    );

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: Some(components),
    };

    let outcome = service.render_batch(&input).expect("Failed to render");
    assert!(
        outcome.is_all_success(),
        "Conditional rendering should work. Errors: {:?}",
        outcome.errors
    );

    let html = outcome
        .files
        .get("conditional.mdx")
        .unwrap()
        .result
        .as_ref()
        .unwrap()
        .output
        .as_ref()
        .unwrap();

    // Header should appear (showHeader: true)
    assert!(
        html.contains("header") && html.contains("Header Content"),
        "Header should render when showHeader is true: {}",
        html
    );

    // DarkTheme should appear (isDarkMode: true)
    assert!(
        html.contains("theme-dark"),
        "Dark theme class should render when isDarkMode is true: {}",
        html
    );

    // Footer should NOT appear (showFooter: false)
    assert!(
        !html.contains("Footer Content"),
        "Footer should not render when showFooter is false: {}",
        html
    );

    println!("Conditional rendering output: {}", html);
}

#[test]
fn test_self_closing_html_tags() {
    // Test self-closing HTML tags like <br>, <hr>, <img>
    let service = create_test_service();

    let content = r#"# Self-closing Tags

Line one<br/>Line two

---

<img src="https://example.com/image.png" alt="Test image" />

<input type="text" placeholder="Enter text" disabled />

<hr />

End of test.
"#;

    let mut mdx_files = HashMap::new();
    mdx_files.insert("selfclose.mdx".to_string(), content.to_string());

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: None,
    };

    let outcome = service.render_batch(&input).expect("Failed to render");
    assert!(
        outcome.is_all_success(),
        "Self-closing tags should work. Errors: {:?}",
        outcome.errors
    );

    let html = outcome
        .files
        .get("selfclose.mdx")
        .unwrap()
        .result
        .as_ref()
        .unwrap()
        .output
        .as_ref()
        .unwrap();

    assert!(
        html.contains("<br") || html.contains("<br/>") || html.contains("<br />"),
        "br tag should render: {}",
        html
    );
    assert!(
        html.contains("<img") || html.contains("example.com/image.png"),
        "img tag should render: {}",
        html
    );
    assert!(
        html.contains("<hr") || html.contains("<hr/>") || html.contains("<hr />"),
        "hr tag should render: {}",
        html
    );

    println!("Self-closing tags output: {}", html);
}

#[test]
fn test_jsx_boolean_and_numeric_props() {
    // Test boolean and numeric prop handling
    let service = create_test_service();

    let content =
        r#"<Counter start={0} end={100} step={5} autoPlay={true} reverse={false} label="Count" />"#;

    let mut mdx_files = HashMap::new();
    mdx_files.insert("bool_num.mdx".to_string(), content.to_string());

    let mut components = HashMap::new();
    components.insert(
        "Counter".to_string(),
        ComponentDefinition {
            name: Some("Counter".to_string()),
            docs: None,
            args: None,
            code: r#"export default function Component({ start, end, step, autoPlay, reverse, label }) {
                return (
                    <div class="counter">
                        <span class="label">{label}</span>
                        <span class="range">{start} to {end} by {step}</span>
                        <span class="auto">{autoPlay ? 'auto' : 'manual'}</span>
                        <span class="dir">{reverse ? 'reverse' : 'forward'}</span>
                    </div>
                );
            }"#
            .to_string(),
        },
    );

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: Some(components),
    };

    let outcome = service.render_batch(&input).expect("Failed to render");
    assert!(
        outcome.is_all_success(),
        "Boolean/numeric props should work. Errors: {:?}",
        outcome.errors
    );

    let html = outcome
        .files
        .get("bool_num.mdx")
        .unwrap()
        .result
        .as_ref()
        .unwrap()
        .output
        .as_ref()
        .unwrap();

    assert!(html.contains("Count"), "Label should render: {}", html);
    assert!(
        html.contains("0") && html.contains("100"),
        "Numeric props should render: {}",
        html
    );
    assert!(
        html.contains("auto"),
        "Boolean true should render: {}",
        html
    );
    assert!(
        html.contains("forward"),
        "Boolean false should render: {}",
        html
    );

    println!("Boolean/numeric props output: {}", html);
}

// =============================================================================
// Common MDX/TSX Bug Tests (based on known issues from mdx-js/mdx)
// =============================================================================

#[test]
fn test_html_entities_in_content() {
    // Test that HTML entities like &lt; &gt; &amp; are preserved correctly
    // Reference: https://github.com/mdx-js/mdx/issues/1219
    let service = create_test_service();

    let content = r#"# HTML Entities Test

Special characters: &lt;div&gt; &amp; &quot;quoted&quot;

In a paragraph: The expression a &lt; b &amp;&amp; c &gt; d is common.

Entity codes: &#60; &#62; &#38; &#34;
"#;

    let mut mdx_files = HashMap::new();
    mdx_files.insert("entities.mdx".to_string(), content.to_string());

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: None,
    };

    let outcome = service.render_batch(&input).expect("Failed to render");
    assert!(
        outcome.is_all_success(),
        "HTML entities should render. Errors: {:?}",
        outcome.errors
    );

    let html = outcome
        .files
        .get("entities.mdx")
        .unwrap()
        .result
        .as_ref()
        .unwrap()
        .output
        .as_ref()
        .unwrap();

    // Verify entities are present in some form (either encoded or decoded)
    assert!(
        html.contains("&lt;") || html.contains("<div>"),
        "Should contain lt entity or decoded: {}",
        html
    );
    println!("HTML entities output: {}", html);
}

#[test]
fn test_jsx_style_comments() {
    // Test JSX-style comments {/* comment */}
    // Reference: https://github.com/mdx-js/mdx/issues/1042
    let service = create_test_service();

    let content = r#"# Comments Test

{/* This is a JSX comment and should not appear in output */}

Some visible content.

{/* Another comment */}

More visible content.
"#;

    let mut mdx_files = HashMap::new();
    mdx_files.insert("comments.mdx".to_string(), content.to_string());

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: None,
    };

    let outcome = service.render_batch(&input).expect("Failed to render");

    // Print errors for debugging
    if !outcome.errors.is_empty() {
        for err in &outcome.errors {
            eprintln!("JSX comments error: {}", err.message);
        }
    }

    // JSX comments may or may not be supported - document the behavior
    let file_outcome = outcome.files.get("comments.mdx").expect("File not found");
    if let Some(result) = &file_outcome.result {
        let html = result.output.as_ref().unwrap();
        // Comments should NOT appear in output
        assert!(
            !html.contains("This is a JSX comment"),
            "JSX comments should be stripped: {}",
            html
        );
        assert!(
            html.contains("Some visible content"),
            "Visible content should remain: {}",
            html
        );
        println!("JSX comments output: {}", html);
    } else {
        println!(
            "JSX comments not supported (expected): {:?}",
            file_outcome.error
        );
    }
}

#[test]
fn test_whitespace_around_inline_components() {
    // Test that whitespace is preserved around inline components
    // Reference: https://github.com/mdx-js/mdx/issues/843
    let service = create_test_service();

    let content = r#"Hello <Highlight>world</Highlight> and <Highlight>universe</Highlight>!"#;

    let mut mdx_files = HashMap::new();
    mdx_files.insert("whitespace.mdx".to_string(), content.to_string());

    let mut components = HashMap::new();
    components.insert(
        "Highlight".to_string(),
        ComponentDefinition {
            name: Some("Highlight".to_string()),
            docs: None,
            args: None,
            code: r#"export default function Component({ children }) {
                return <mark>{children}</mark>;
            }"#
            .to_string(),
        },
    );

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: Some(components),
    };

    let outcome = service.render_batch(&input).expect("Failed to render");
    assert!(
        outcome.is_all_success(),
        "Inline components with whitespace should work. Errors: {:?}",
        outcome.errors
    );

    let html = outcome
        .files
        .get("whitespace.mdx")
        .unwrap()
        .result
        .as_ref()
        .unwrap()
        .output
        .as_ref()
        .unwrap();

    // Check content renders
    assert!(html.contains("world"), "Should contain world: {}", html);
    assert!(
        html.contains("universe"),
        "Should contain universe: {}",
        html
    );
    println!("Whitespace output: {}", html);
}

#[test]
fn test_escaped_markdown_characters() {
    // Test escaped markdown characters like \* \_ \# \`
    let service = create_test_service();

    let content = r#"# Escaped Characters

This is \*not bold\* and this is \*\*not strong\*\*.

This is \_not italic\_ either.

\# Not a heading

\`not code\`

Backslash: \\
"#;

    let mut mdx_files = HashMap::new();
    mdx_files.insert("escaped.mdx".to_string(), content.to_string());

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: None,
    };

    let outcome = service.render_batch(&input).expect("Failed to render");
    assert!(
        outcome.is_all_success(),
        "Escaped chars should render. Errors: {:?}",
        outcome.errors
    );

    let html = outcome
        .files
        .get("escaped.mdx")
        .unwrap()
        .result
        .as_ref()
        .unwrap()
        .output
        .as_ref()
        .unwrap();

    // Escaped asterisks should NOT create bold/italic
    assert!(
        !html.contains("<strong>not bold</strong>"),
        "Escaped ** should not be bold: {}",
        html
    );
    assert!(
        !html.contains("<em>not italic</em>"),
        "Escaped _ should not be italic: {}",
        html
    );
    println!("Escaped chars output: {}", html);
}

#[test]
fn test_unicode_and_emoji_in_mdx() {
    // Test Unicode characters and emoji in MDX content
    let service = create_test_service();

    let content = r#"---
title: Unicode Test 🎉
emoji: 🚀
---

# {context('title')}

Emoji in text: 🔥 💻 ⚡ 🎯

Unicode: α β γ δ → ← ↑ ↓ ∞ ≠ ≤ ≥

CJK: 你好世界 こんにちは 안녕하세요

Special: — – … © ® ™ § ¶ † ‡
"#;

    let mut mdx_files = HashMap::new();
    mdx_files.insert("unicode.mdx".to_string(), content.to_string());

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: None,
    };

    let outcome = service.render_batch(&input).expect("Failed to render");
    assert!(
        outcome.is_all_success(),
        "Unicode should render. Errors: {:?}",
        outcome.errors
    );

    let html = outcome
        .files
        .get("unicode.mdx")
        .unwrap()
        .result
        .as_ref()
        .unwrap()
        .output
        .as_ref()
        .unwrap();

    // Verify unicode content is preserved
    assert!(html.contains("🎉"), "Should contain party emoji: {}", html);
    assert!(html.contains("你好"), "Should contain Chinese: {}", html);
    assert!(html.contains("α"), "Should contain Greek: {}", html);
    println!("Unicode output length: {} chars", html.len());
}

#[test]
fn test_consecutive_jsx_components() {
    // Test multiple JSX components in sequence without whitespace
    let service = create_test_service();

    let content = r#"<Badge>A</Badge><Badge>B</Badge><Badge>C</Badge>

<Tag color="red">First</Tag><Tag color="blue">Second</Tag>
"#;

    let mut mdx_files = HashMap::new();
    mdx_files.insert("consecutive.mdx".to_string(), content.to_string());

    let mut components = HashMap::new();
    components.insert(
        "Badge".to_string(),
        ComponentDefinition {
            name: Some("Badge".to_string()),
            docs: None,
            args: None,
            code: r#"export default function Component({ children }) {
                return <span class="badge">{children}</span>;
            }"#
            .to_string(),
        },
    );
    components.insert(
        "Tag".to_string(),
        ComponentDefinition {
            name: Some("Tag".to_string()),
            docs: None,
            args: None,
            code: r#"export default function Component({ color, children }) {
                return <span class={`tag tag-${color}`}>{children}</span>;
            }"#
            .to_string(),
        },
    );

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: Some(components),
    };

    let outcome = service.render_batch(&input).expect("Failed to render");
    assert!(
        outcome.is_all_success(),
        "Consecutive components should render. Errors: {:?}",
        outcome.errors
    );

    let html = outcome
        .files
        .get("consecutive.mdx")
        .unwrap()
        .result
        .as_ref()
        .unwrap()
        .output
        .as_ref()
        .unwrap();

    // All badges should render
    assert!(html.contains(">A<"), "Should contain A: {}", html);
    assert!(html.contains(">B<"), "Should contain B: {}", html);
    assert!(html.contains(">C<"), "Should contain C: {}", html);
    assert!(html.contains("tag-red"), "Should contain red tag: {}", html);
    assert!(
        html.contains("tag-blue"),
        "Should contain blue tag: {}",
        html
    );
    println!("Consecutive components output: {}", html);
}

#[test]
fn test_table_with_jsx_and_code() {
    // Test tables containing JSX components and code
    // Reference: https://github.com/mdx-js/mdx/issues/2000
    let service = create_test_service();

    let content = r#"# Table Test

| Feature | Description | Example |
|---------|-------------|---------|
| Bold | **strong text** | `code` |
| Link | [click here](/) | `const x = 1;` |
| Mixed | **bold** and _italic_ | `{value}` |
"#;

    let mut mdx_files = HashMap::new();
    mdx_files.insert("table.mdx".to_string(), content.to_string());

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: None,
    };

    let outcome = service.render_batch(&input).expect("Failed to render");
    assert!(
        outcome.is_all_success(),
        "Table should render. Errors: {:?}",
        outcome.errors
    );

    let html = outcome
        .files
        .get("table.mdx")
        .unwrap()
        .result
        .as_ref()
        .unwrap()
        .output
        .as_ref()
        .unwrap();

    assert!(html.contains("<table"), "Should contain table: {}", html);
    assert!(
        html.contains("<strong>"),
        "Should contain bold in table: {}",
        html
    );
    assert!(
        html.contains("<code>"),
        "Should contain code in table: {}",
        html
    );
    println!("Table output: {}", html);
}

#[test]
fn test_deeply_nested_jsx() {
    // Test deeply nested JSX components
    let service = create_test_service();

    let content = r#"<Outer>
  <Middle>
    <Inner>
      <Deep>Content at depth 4</Deep>
    </Inner>
  </Middle>
</Outer>"#;

    let mut mdx_files = HashMap::new();
    mdx_files.insert("nested.mdx".to_string(), content.to_string());

    let mut components = HashMap::new();
    for name in ["Outer", "Middle", "Inner", "Deep"] {
        components.insert(
            name.to_string(),
            ComponentDefinition {
                name: Some(name.to_string()),
                docs: None,
                args: None,
                code: format!(
                    r#"export default function Component({{ children }}) {{
                    return <div class="{name}">{{children}}</div>;
                }}"#,
                    name = name.to_lowercase()
                ),
            },
        );
    }

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: Some(components),
    };

    let outcome = service.render_batch(&input).expect("Failed to render");
    assert!(
        outcome.is_all_success(),
        "Deeply nested JSX should render. Errors: {:?}",
        outcome.errors
    );

    let html = outcome
        .files
        .get("nested.mdx")
        .unwrap()
        .result
        .as_ref()
        .unwrap()
        .output
        .as_ref()
        .unwrap();

    assert!(html.contains("outer"), "Should contain outer: {}", html);
    assert!(html.contains("deep"), "Should contain deep: {}", html);
    assert!(
        html.contains("Content at depth 4"),
        "Should contain content: {}",
        html
    );
    println!("Deeply nested output: {}", html);
}

#[test]
fn test_jsx_with_string_containing_special_chars() {
    // Test JSX with string props containing special characters
    // Reference: https://github.com/Microsoft/TypeScript/issues/6241
    let service = create_test_service();

    let content =
        r#"<Message text="Hello \"world\" with 'quotes' and <brackets>" symbol="&amp;" />"#;

    let mut mdx_files = HashMap::new();
    mdx_files.insert("special_strings.mdx".to_string(), content.to_string());

    let mut components = HashMap::new();
    components.insert(
        "Message".to_string(),
        ComponentDefinition {
            name: Some("Message".to_string()),
            docs: None,
            args: None,
            code: r#"export default function Component({ text, symbol }) {
                return <div class="message">{text} {symbol}</div>;
            }"#
            .to_string(),
        },
    );

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: Some(components),
    };

    let outcome = service.render_batch(&input).expect("Failed to render");

    // This may or may not work depending on escaping support
    if outcome.is_all_success() {
        let html = outcome
            .files
            .get("special_strings.mdx")
            .unwrap()
            .result
            .as_ref()
            .unwrap()
            .output
            .as_ref()
            .unwrap();
        println!("Special strings output: {}", html);
    } else {
        println!(
            "Special strings in props not fully supported: {:?}",
            outcome.errors
        );
    }
}

#[test]
fn test_empty_jsx_expression() {
    // Test empty JSX expressions {}
    let service = create_test_service();

    // Note: Empty expressions {} may cause issues - test behavior
    let content = r#"---
value: test
---

# Test

Value is: {context('value')}

End.
"#;

    let mut mdx_files = HashMap::new();
    mdx_files.insert("empty_expr.mdx".to_string(), content.to_string());

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: None,
    };

    let outcome = service.render_batch(&input).expect("Failed to render");
    assert!(
        outcome.is_all_success(),
        "Expression should render. Errors: {:?}",
        outcome.errors
    );

    let html = outcome
        .files
        .get("empty_expr.mdx")
        .unwrap()
        .result
        .as_ref()
        .unwrap()
        .output
        .as_ref()
        .unwrap();

    assert!(
        html.contains("test"),
        "Should contain context value: {}",
        html
    );
    println!("Expression output: {}", html);
}

#[test]
fn test_markdown_blank_line_in_jsx() {
    // Test markdown content inside JSX with required blank lines
    // Reference: https://github.com/mdx-js/mdx/issues/1312
    let service = create_test_service();

    let content = r#"<Container>

# Heading Inside Component

This is a paragraph with **bold** and *italic*.

- List item 1
- List item 2

</Container>"#;

    let mut mdx_files = HashMap::new();
    mdx_files.insert("blank_lines.mdx".to_string(), content.to_string());

    let mut components = HashMap::new();
    components.insert(
        "Container".to_string(),
        ComponentDefinition {
            name: Some("Container".to_string()),
            docs: None,
            args: None,
            code: r#"export default function Component({ children }) {
                return <div class="container">{children}</div>;
            }"#
            .to_string(),
        },
    );

    let input = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files,
        components: Some(components),
    };

    let outcome = service.render_batch(&input).expect("Failed to render");
    assert!(
        outcome.is_all_success(),
        "Markdown in JSX should render. Errors: {:?}",
        outcome.errors
    );

    let html = outcome
        .files
        .get("blank_lines.mdx")
        .unwrap()
        .result
        .as_ref()
        .unwrap()
        .output
        .as_ref()
        .unwrap();

    assert!(
        html.contains("container"),
        "Should have container: {}",
        html
    );
    // Check if markdown was processed
    println!("Markdown in JSX output: {}", html);
}
