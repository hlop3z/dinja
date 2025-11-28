//! Benchmarks for MDX rendering performance
//!
//! Run with: cargo bench -p dinja-core
//!
//! Note: Due to V8 isolate constraints, benchmarks use a single shared service.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use dinja_core::models::{ComponentDefinition, NamedMdxBatchInput, OutputFormat, RenderSettings};
use dinja_core::service::{RenderService, RenderServiceConfig};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

static SERVICE: OnceLock<RenderService> = OnceLock::new();

fn get_service() -> &'static RenderService {
    SERVICE.get_or_init(|| {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let static_dir = PathBuf::from(manifest_dir).join("static");

        let config = RenderServiceConfig {
            static_dir,
            max_cached_renderers: 4,
            resource_limits: Default::default(),
        };

        RenderService::new(config).expect("Failed to create RenderService")
    })
}

fn simple_markdown() -> String {
    r#"---
title: Hello World
---

# Welcome

This is a **simple** markdown file with *italic* and `code`.

- Item 1
- Item 2
- Item 3
"#
    .to_string()
}

fn markdown_with_component() -> (String, HashMap<String, ComponentDefinition>) {
    let mdx = r#"---
title: Component Test
author: Test
---

# Welcome

<Card title={context('title')} author={context('author')}>
  This is the card content with **bold** text.
</Card>

Some text after the component.
"#
    .to_string();

    let mut components = HashMap::new();
    components.insert(
        "Card".to_string(),
        ComponentDefinition {
            name: Some("Card".to_string()),
            docs: None,
            args: None,
            code: r#"export default function Component({ title, author, children }) {
    return (
        <div class="card">
            <h2>{title}</h2>
            <span class="author">{author}</span>
            <div class="content">{children}</div>
        </div>
    );
}"#
            .to_string(),
        },
    );

    (mdx, components)
}

fn generate_components(count: usize) -> (String, HashMap<String, ComponentDefinition>) {
    let mut mdx = String::from("---\ntitle: Stress Test\n---\n\n# Components\n\n");
    let mut components = HashMap::new();

    for i in 0..count {
        let name = format!("Comp{}", i);
        mdx.push_str(&format!("<{name}>Content {i}</{name}>\n\n"));

        components.insert(
            name.clone(),
            ComponentDefinition {
                name: Some(name),
                docs: None,
                args: None,
                code: format!(
                    r#"export default function Component({{ children }}) {{
    return <div class="comp-{i}">{{children}}</div>;
}}"#
                ),
            },
        );
    }

    (mdx, components)
}

fn render_benchmarks(c: &mut Criterion) {
    let service = get_service();

    // Simple markdown benchmark
    let content = simple_markdown();
    c.bench_function("simple_markdown", |b| {
        b.iter(|| {
            let mut mdx_files = HashMap::new();
            mdx_files.insert("test.mdx".to_string(), content.clone());

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

            black_box(service.render_batch(&input).unwrap())
        })
    });

    // Markdown with component benchmark
    let (content, components) = markdown_with_component();
    c.bench_function("with_component", |b| {
        b.iter(|| {
            let mut mdx_files = HashMap::new();
            mdx_files.insert("test.mdx".to_string(), content.clone());

            let input = NamedMdxBatchInput {
                settings: RenderSettings {
                    output: OutputFormat::Html,
                    minify: false,
                    utils: None,
                    directives: None,
                },
                mdx: mdx_files,
                components: Some(components.clone()),
            };

            black_box(service.render_batch(&input).unwrap())
        })
    });

    // Component scaling benchmark
    let mut group = c.benchmark_group("component_count");
    for count in [10, 50, 100].iter() {
        let (content, components) = generate_components(*count);

        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, _| {
            b.iter(|| {
                let mut mdx_files = HashMap::new();
                mdx_files.insert("test.mdx".to_string(), content.clone());

                let input = NamedMdxBatchInput {
                    settings: RenderSettings {
                        output: OutputFormat::Html,
                        minify: false,
                        utils: None,
                        directives: None,
                    },
                    mdx: mdx_files,
                    components: Some(components.clone()),
                };

                black_box(service.render_batch(&input).unwrap())
            })
        });
    }
    group.finish();

    // Output format benchmark
    let (content, components) = markdown_with_component();
    let mut group = c.benchmark_group("output_format");
    for (name, format) in [
        ("html", OutputFormat::Html),
        ("javascript", OutputFormat::Javascript),
        ("schema", OutputFormat::Schema),
    ] {
        group.bench_with_input(BenchmarkId::from_parameter(name), &format, |b, fmt| {
            b.iter(|| {
                let mut mdx_files = HashMap::new();
                mdx_files.insert("test.mdx".to_string(), content.clone());

                let input = NamedMdxBatchInput {
                    settings: RenderSettings {
                        output: fmt.clone(),
                        minify: false,
                        utils: None,
                        directives: None,
                    },
                    mdx: mdx_files,
                    components: Some(components.clone()),
                };

                black_box(service.render_batch(&input).unwrap())
            })
        });
    }
    group.finish();

    // Batch size benchmark
    let content = simple_markdown();
    let mut group = c.benchmark_group("batch_size");
    for count in [1, 5, 10, 25].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &size| {
            b.iter(|| {
                let mut mdx_files = HashMap::new();
                for i in 0..size {
                    mdx_files.insert(format!("file{}.mdx", i), content.clone());
                }

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

                black_box(service.render_batch(&input).unwrap())
            })
        });
    }
    group.finish();
}

criterion_group!(benches, render_benchmarks);
criterion_main!(benches);
