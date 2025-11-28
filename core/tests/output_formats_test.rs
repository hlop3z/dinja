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
