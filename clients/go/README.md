# Dinja Go Client

Go HTTP client for the [Dinja](https://github.com/hlop3z/dinja) MDX rendering service.

## Installation

```bash
go get github.com/hlop3z/dinja-go
```

## Requirements

- Go 1.21+
- Running Dinja service (default: `http://localhost:8080`)

## Quick Start

```go
package main

import (
    "context"
    "fmt"
    "log"

    dinja "github.com/hlop3z/dinja-go"
)

func main() {
    // Create a renderer (connects to localhost:8080 by default)
    renderer := dinja.New()

    // Render MDX to HTML
    result, err := renderer.HTML(context.Background(), dinja.Input{
        Views: map[string]string{
            "page.mdx": "# Hello World\n\nThis is **MDX** content.",
        },
    })
    if err != nil {
        log.Fatal(err)
    }

    // Get the rendered output
    fmt.Println(result.GetOutput("page.mdx"))
}
```

## Configuration

```go
import "time"

// Custom base URL
renderer := dinja.New(dinja.WithBaseURL("http://dinja:9000"))

// Custom timeout
renderer := dinja.New(dinja.WithTimeout(60 * time.Second))

// Custom HTTP client
client := &http.Client{
    Timeout: 30 * time.Second,
    Transport: &http.Transport{
        MaxIdleConns: 10,
    },
}
renderer := dinja.New(dinja.WithHTTPClient(client))

// Combine options
renderer := dinja.New(
    dinja.WithBaseURL("http://dinja:9000"),
    dinja.WithTimeout(60 * time.Second),
)
```

## Output Formats

```go
ctx := context.Background()
input := dinja.Input{Views: map[string]string{"page.mdx": "# Hello"}}

// Render to HTML
result, _ := renderer.HTML(ctx, input)

// Render to JavaScript
result, _ := renderer.JavaScript(ctx, input)

// Extract component schema
result, _ := renderer.Schema(ctx, input)

// Render to JSON tree
result, _ := renderer.JSON(ctx, input)

// Generic render with output type
result, _ := renderer.Render(ctx, dinja.OutputHTML, input)
```

## Custom Components

```go
result, err := renderer.HTML(ctx, dinja.Input{
    Views: map[string]string{
        "page.mdx": "<Button>Click me</Button>",
    },
    Components: map[string]dinja.Component{
        "Button": {
            Code: `export default function Button({children}) {
                return <button className="btn">{children}</button>;
            }`,
            Name: "Button",
            Docs: "A styled button component",
        },
    },
})
```

## Global Utilities

```go
result, err := renderer.HTML(ctx, dinja.Input{
    Views: map[string]string{
        "page.mdx": "<Greeting />",
    },
    Components: map[string]dinja.Component{
        "Greeting": {
            Code: `export default function Greeting() {
                return <h1>{utils.message}</h1>;
            }`,
        },
    },
    Utils: `export default { message: "Hello from utils!" }`,
})
```

## Batch Rendering

```go
result, err := renderer.HTML(ctx, dinja.Input{
    Views: map[string]string{
        "index.mdx":   "# Home",
        "about.mdx":   "# About Us",
        "contact.mdx": "# Contact",
    },
})

fmt.Printf("Total: %d, Succeeded: %d, Failed: %d\n",
    result.Total, result.Succeeded, result.Failed)

// Iterate over results
for filename, fileResult := range result.Files {
    if fileResult.Success {
        fmt.Printf("%s: %s\n", filename, result.GetOutput(filename))
    } else {
        fmt.Printf("%s: ERROR - %s\n", filename, fileResult.Error)
    }
}
```

## Builder Pattern

```go
opts := dinja.NewRenderOptions().
    WithView("page.mdx", "# Hello").
    WithView("about.mdx", "# About").
    WithComponentCode("Button", "export default ({children}) => <button>{children}</button>").
    WithUtils("export default { version: '1.0' }").
    WithMinify(false).
    WithDirectives("v-", "@", "x-")

result, err := renderer.HTML(ctx, opts.Build())
```

## Error Handling

```go
result, err := renderer.HTML(ctx, input)
if err != nil {
    // HTTP or connection error
    log.Fatalf("Request failed: %v", err)
}

// Check for partial failures
if !result.IsAllSuccess() {
    for _, errInfo := range result.Errors {
        log.Printf("File %s failed: %s", errInfo.File, errInfo.Message)
    }
}

// Get error for specific file
if errMsg := result.GetError("page.mdx"); errMsg != "" {
    log.Printf("page.mdx error: %s", errMsg)
}
```

## Health Check

```go
healthy, err := renderer.Health(ctx)
if err != nil {
    log.Fatalf("Health check failed: %v", err)
}
if !healthy {
    log.Println("Service is not healthy")
}
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
go func() {
    time.Sleep(2 * time.Second)
    cancel() // Cancel after 2 seconds
}()

result, err := renderer.HTML(ctx, input)
```

## API Reference

### Types

```go
// Renderer is the HTTP client
type Renderer struct { ... }

// Input is the render request
type Input struct {
    Views      map[string]string       // MDX content (required)
    Components map[string]Component    // Custom components
    Utils      string                  // Global utilities
    Minify     *bool                   // Enable minification (default: true)
    Directives []string                // Directive prefixes for schema
}

// Component definition
type Component struct {
    Code string // JSX/TSX code (required)
    Name string // Component name
    Docs string // Documentation
    Args any    // Props type info
}

// Result is the batch response
type Result struct {
    Total     int                     // Total files
    Succeeded int                     // Successful renders
    Failed    int                     // Failed renders
    Files     map[string]FileResult   // Per-file results
    Errors    []ErrorInfo             // Error list
}

// Result methods
func (r *Result) IsAllSuccess() bool
func (r *Result) GetOutput(filename string) string
func (r *Result) GetMetadata(filename string) map[string]any
func (r *Result) GetError(filename string) string
```

## Running the Dinja Service

The client requires a running Dinja service. Start it with Docker:

```bash
docker run -p 8080:8080 hlop3z/dinja
```

Or build from source:

```bash
cargo run --release -p dinja-server
```

## License

MIT
