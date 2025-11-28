# Dinja Go Client

HTTP client for the Dinja MDX rendering service.

## Installation

```bash
go get github.com/hlop3z/dinja/clients/go
```

## Requirements

Start the Dinja service via Docker:

```bash
docker pull ghcr.io/hlop3z/dinja:latest
docker run -p 8080:8080 ghcr.io/hlop3z/dinja:latest
```

## Usage

### Basic Example

```go
package main

import (
    "context"
    "fmt"
    "log"

    dinja "github.com/hlop3z/dinja/clients/go"
)

func main() {
    // Connect to the service
    renderer := dinja.New()

    // Check health
    ctx := context.Background()
    if healthy, _ := renderer.Health(ctx); healthy {
        fmt.Println("Service is running!")
    }

    // Render MDX to HTML
    result, err := renderer.HTML(ctx, dinja.Input{
        Views: map[string]string{
            "page.mdx": "# Hello World\n\nThis is **bold** text.",
        },
        Utils: "export default { greeting: 'Hello' }",
    })
    if err != nil {
        log.Fatal(err)
    }

    // Get the output
    fmt.Println(result.GetOutput("page.mdx"))
    // Output: <h1>Hello World</h1><p>This is <strong>bold</strong> text.</p>
}
```

### Render Methods

```go
ctx := context.Background()
input := dinja.Input{Views: map[string]string{"page.mdx": "# Hello"}}

// Render to HTML
result, _ := renderer.HTML(ctx, input)

// Render to JavaScript
result, _ := renderer.JavaScript(ctx, input)

// Extract schema (component names)
result, _ := renderer.Schema(ctx, input)

// Render to JSON tree
result, _ := renderer.JSON(ctx, input)

// Generic render with output format
result, _ := renderer.Render(ctx, dinja.OutputHTML, input)
```

### Components

```go
result, err := renderer.HTML(ctx, dinja.Input{
    Views: map[string]string{
        "app.mdx": "# App\n\n<Button>Click me</Button>",
    },
    Components: map[string]dinja.Component{
        "Button": {
            Code: "export default function Component(props) { return <button>{props.children}</button>; }",
        },
    },
})
```

### Input Options

All render methods accept an `Input` struct with:

- `Views`: Map of view names to MDX content (required)
- `Components`: Map of component names to Component definitions (optional)
- `Utils`: JavaScript utilities code (optional)
- `Minify`: Enable minification (default: true)
- `Directives`: Slice of directive prefixes for schema extraction (optional)

### Result Object

```go
result, _ := renderer.HTML(ctx, input)

// Check success
result.IsAllSuccess()  // true if all files succeeded

// Get output for a file
result.GetOutput("page.mdx")

// Get metadata for a file
result.GetMetadata("page.mdx")

// Access individual files
result.Files["page.mdx"].Success
result.Files["page.mdx"].Result.Output
result.Files["page.mdx"].Result.Metadata
result.Files["page.mdx"].Error  // If failed
```

## API Reference

### Configuration

```go
import "time"

// Custom base URL
renderer := dinja.New(dinja.WithBaseURL("http://dinja:9000"))

// Custom timeout
renderer := dinja.New(dinja.WithTimeout(60 * time.Second))

// Custom HTTP client
client := &http.Client{Timeout: 30 * time.Second}
renderer := dinja.New(dinja.WithHTTPClient(client))

// Combine options
renderer := dinja.New(
    dinja.WithBaseURL("http://dinja:9000"),
    dinja.WithTimeout(60 * time.Second),
)
```

### Types

```go
import dinja "github.com/hlop3z/dinja/clients/go"

// Renderer - HTTP client
type Renderer struct { ... }

// Input - Render request
type Input struct {
    Views      map[string]string       // MDX content (required)
    Components map[string]Component    // Custom components
    Utils      string                  // Global utilities
    Minify     *bool                   // Enable minification (default: true)
    Directives []string                // Directive prefixes for schema
}

// Component - Component definition
type Component struct {
    Code string // JSX/TSX code (required)
    Name string // Component name
    Docs string // Documentation
    Args any    // Props type info
}

// Result - Batch response
type Result struct {
    Total     int                     // Total files
    Succeeded int                     // Successful renders
    Failed    int                     // Failed renders
    Files     map[string]FileResult   // Per-file results
    Errors    []ErrorInfo             // Error list
}

// Output - Format type: OutputHTML, OutputJavaScript, OutputSchema, OutputJSON
type Output string
```

### Helper Methods

```go
// Result methods
func (r *Result) IsAllSuccess() bool
func (r *Result) GetOutput(filename string) string
func (r *Result) GetMetadata(filename string) map[string]any
func (r *Result) GetError(filename string) string
```

## Context Support

All methods accept a `context.Context` for cancellation and timeouts:

```go
// With timeout
ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
defer cancel()
result, err := renderer.HTML(ctx, input)

// With cancellation
ctx, cancel := context.WithCancel(context.Background())
defer cancel()
result, err := renderer.HTML(ctx, input)
```

## License

BSD-3-Clause

## Links

- [GitHub](https://github.com/hlop3z/dinja)
- [Documentation](https://hlop3z.github.io/dinja)
- [Go Package](https://pkg.go.dev/github.com/hlop3z/dinja/clients/go)
