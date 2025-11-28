# @dinja/core

HTTP client for the Dinja MDX rendering service.

## Installation

```bash
npm install @dinja/core
# or
yarn add @dinja/core
# or
pnpm add @dinja/core
```

## Requirements

Start the Dinja service via Docker:

```bash
docker pull ghcr.io/hlop3z/dinja:latest
docker run -p 8080:8080 ghcr.io/hlop3z/dinja:latest
```

## Usage

### Basic Example

```typescript
import { Renderer, isAllSuccess, getOutput } from '@dinja/core';

// Connect to the service
const renderer = new Renderer({ baseUrl: 'http://localhost:8080' });

// Check health
if (await renderer.health()) {
  console.log('Service is running!');
}

// Render MDX to HTML
const result = await renderer.html({
  views: { 'page.mdx': '# Hello World\n\nThis is **bold** text.' },
  utils: "export default { greeting: 'Hello' }",
});

// Get the output
console.log(getOutput(result, 'page.mdx'));
// Output: <h1>Hello World</h1><p>This is <strong>bold</strong> text.</p>
```

### Render Methods

```typescript
// Render to HTML
const result = await renderer.html({ views: {...} });

// Render to JavaScript
const result = await renderer.javascript({ views: {...} });

// Extract schema (component names)
const result = await renderer.schema({ views: {...} });

// Render to JSON tree
const result = await renderer.json({ views: {...} });

// Generic render with output format
const result = await renderer.render('html', { views: {...} });
```

### Components

```typescript
const result = await renderer.html({
  views: { 'app.mdx': '# App\n\n<Button>Click me</Button>' },
  components: {
    Button: 'function Component(props) { return <button>{props.children}</button>; }',
  },
});
```

### Input Options

All render methods accept an `Input` object with:

- `views`: Record mapping view names to MDX content (required)
- `components`: Record mapping component names to code (optional)
- `utils`: JavaScript utilities code (optional)
- `minify`: Enable minification (default: true)
- `directives`: Array of directive prefixes for schema extraction (optional)

### Result Object

```typescript
const result = await renderer.html({ views: {...} });

// Check success
isAllSuccess(result);  // true if all files succeeded

// Get output for a file
getOutput(result, 'page.mdx');

// Get metadata for a file
getMetadata(result, 'page.mdx');

// Access individual files
result.files['page.mdx'].success;
result.files['page.mdx'].result?.output;
result.files['page.mdx'].result?.metadata;
result.files['page.mdx'].error;  // If failed
```

## API Reference

### Types

```typescript
import {
  Renderer,           // HTTP client class
  Input,              // Input interface
  Result,             // Batch result interface
  FileResult,         // Individual file result
  Component,          // Component definition
  Output,             // Type: "html" | "javascript" | "schema" | "json"
  RendererConfig,     // Renderer configuration
  isAllSuccess,       // Helper function
  getOutput,          // Helper function
  getMetadata,        // Helper function
} from '@dinja/core';
```

### Renderer Configuration

```typescript
const renderer = new Renderer({
  baseUrl: 'http://localhost:8080',  // Default
  timeout: 30000,                    // Timeout in milliseconds (default: 30000)
});
```

## Platform Support

Works in any JavaScript runtime that supports `fetch`:
- Node.js >= 18
- Deno
- Bun
- Browser (with CORS support)

## License

BSD-3-Clause

## Links

- [GitHub](https://github.com/hlop3z/dinja)
- [Documentation](https://hlop3z.github.io/dinja)
- [npm](https://www.npmjs.com/package/@dinja/core)
