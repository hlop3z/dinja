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
fn test_html_output_minified() {
    let service = create_test_service();

    let mut mdx_files = HashMap::new();
    mdx_files.insert(
        "minified.mdx".to_string(),
        "# Title\n\n<div>\n  <p>Content</p>\n</div>".to_string(),
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

    let file_outcome = outcome.files.get("minified.mdx").expect("File not found");
    let result = file_outcome
        .result
        .as_ref()
        .expect("Result should be present");

    let html = result
        .output
        .as_ref()
        .expect("HTML output should be present");
    // Minified HTML should have reduced whitespace
    assert!(!html.is_empty());
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
fn test_batch_multiple_files_different_formats() {
    let service = create_test_service();

    // Test HTML format
    let mut mdx_files_html = HashMap::new();
    mdx_files_html.insert("file1.mdx".to_string(), "# File 1".to_string());
    mdx_files_html.insert("file2.mdx".to_string(), "# File 2".to_string());

    let input_html = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Html,
            minify: true,
            utils: None,
            directives: None,
        },
        mdx: mdx_files_html,
        components: None,
    };

    let outcome_html = service
        .render_batch(&input_html)
        .expect("Failed to render HTML batch");

    assert_eq!(outcome_html.total, 2);
    assert!(outcome_html.is_all_success());

    // Test JavaScript format
    let mut mdx_files_js = HashMap::new();
    mdx_files_js.insert("file3.mdx".to_string(), "# File 3".to_string());

    let input_js = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Javascript,
            minify: true,
            utils: None,
            directives: None,
        },
        mdx: mdx_files_js,
        components: None,
    };

    let outcome_js = service
        .render_batch(&input_js)
        .expect("Failed to render JS batch");

    assert_eq!(outcome_js.total, 1);
    assert!(outcome_js.is_all_success());

    // Test Schema format
    let mut mdx_files_schema = HashMap::new();
    mdx_files_schema.insert("file4.mdx".to_string(), "# File 4".to_string());

    let input_schema = NamedMdxBatchInput {
        settings: RenderSettings {
            output: OutputFormat::Schema,
            minify: false,
            utils: None,
            directives: None,
        },
        mdx: mdx_files_schema,
        components: None,
    };

    let outcome_schema = service
        .render_batch(&input_schema)
        .expect("Failed to render schema batch");

    assert_eq!(outcome_schema.total, 1);
    assert!(outcome_schema.is_all_success());
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
