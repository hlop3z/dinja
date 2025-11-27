# @dinja/core

Fast MDX renderer with component support - JavaScript bindings powered by a Rust core.

## Installation

```bash
npm install @dinja/core
# or
yarn add @dinja/core
# or
pnpm add @dinja/core
```

## Usage

### Basic Example

```javascript
import { Renderer } from '@dinja/core';

// Create a renderer instance (engine loads once)
const renderer = new Renderer();

// Render MDX content
const result = renderer.render({
  settings: {
    output: 'html',
    minify: false
  },
  mdx: {
    'example.mdx': '# Hello **dinja**'
  }
});

console.log(result);
// {
//   total: 1,
//   succeeded: 1,
//   failed: 0,
//   errors: [],
//   files: {
//     'example.mdx': {
//       success: true,
//       output: '<h1>Hello <strong>dinja</strong></h1>'
//     }
//   }
// }
```

### TypeScript Example

```typescript
import { Renderer, RenderInput, RenderResult } from '@dinja/core';

const renderer = new Renderer();

const input: RenderInput = {
  settings: {
    output: 'html',
    minify: true
  },
  mdx: {
    'page.mdx': '# Welcome to dinja'
  }
};

const result: RenderResult = renderer.render(input);

if (result.files['page.mdx'].success) {
  console.log(result.files['page.mdx'].output);
}
```

### Output Formats

The renderer supports multiple output formats:

```javascript
import { Renderer } from '@dinja/core';

const renderer = new Renderer();

// HTML output
renderer.render({
  settings: { output: 'html', minify: false },
  mdx: { 'file.mdx': '# Hello' }
});

// JavaScript output (executable code)
renderer.render({
  settings: { output: 'javascript', minify: false },
  mdx: { 'file.mdx': '# Hello' }
});

// Schema output (AST)
renderer.render({
  settings: { output: 'schema', minify: false },
  mdx: { 'file.mdx': '# Hello' }
});

// JSON output (schema as JSON string)
renderer.render({
  settings: { output: 'json', minify: false },
  mdx: { 'file.mdx': '# Hello' }
});
```

### Custom Components

```javascript
import { Renderer } from '@dinja/core';

const renderer = new Renderer();

const result = renderer.render({
  settings: { output: 'html', minify: false },
  mdx: {
    'app.mdx': `
import { Button } from './button';

<Button>Click me</Button>
    `
  },
  components: {
    Button: {
      name: 'Button',
      code: `
export default function Component({ children }) {
  return <button class="custom-btn">{children}</button>;
}
      `
    }
  }
});
```

### Batch Rendering

The renderer efficiently handles multiple files in a single call:

```javascript
import { Renderer } from '@dinja/core';

const renderer = new Renderer();

const result = renderer.render({
  settings: { output: 'html', minify: false },
  mdx: {
    'page1.mdx': '# Page 1',
    'page2.mdx': '# Page 2',
    'page3.mdx': '# Page 3'
  }
});

console.log(`Rendered ${result.succeeded} out of ${result.total} files`);

// Access individual results
for (const [filename, outcome] of Object.entries(result.files)) {
  if (outcome.success) {
    console.log(`${filename}: ${outcome.output}`);
  } else {
    console.error(`${filename} failed: ${outcome.error}`);
  }
}
```

### Reusable Renderer Instance

The `Renderer` class maintains a single render service instance and reuses it across multiple renders, which prevents V8 isolate issues and improves performance:

```javascript
import { Renderer } from '@dinja/core';

const renderer = new Renderer();

// First render with HTML output
const html = renderer.render({
  settings: { output: 'html', minify: false },
  mdx: { 'page.mdx': '# Hello' }
});

// Second render with schema output (reuses same instance)
const schema = renderer.render({
  settings: { output: 'schema', minify: false },
  mdx: { 'page.mdx': '# World' }
});
```

### Using Global Utils

Inject global JavaScript utilities available in all components:

```javascript
import { Renderer } from '@dinja/core';

const renderer = new Renderer();

const result = renderer.render({
  settings: {
    output: 'html',
    minify: false,
    utils: "export default { greeting: 'Hello', emoji: '👋' }"
  },
  mdx: {
    'page.mdx': '<Greeting name="Alice" />'
  },
  components: {
    Greeting: {
      name: 'Greeting',
      code: `
        export default function Component(props) {
          return <div>{utils.greeting} {props.name} {utils.emoji}</div>;
        }
      `
    }
  }
});
```

The `utils` object must be exported using `export default { ... }` and will be available globally as `utils` in all component code. Invalid utils code is silently ignored.

## API Reference

### `Renderer`

#### Constructor

```typescript
new Renderer()
```

Creates a new Renderer instance. The engine is loaded once during initialization and reused for all subsequent renders.

#### `render(input: RenderInput): RenderResult`

Renders MDX content.

**Parameters:**
- `input.settings` - Render settings
  - `output`: `'html' | 'javascript' | 'schema' | 'json'` - Output format
  - `minify`: `boolean` - Whether to minify the output
  - `utils`: `string` (optional) - JavaScript snippet to inject as global utilities (must use `export default { ... }`)
- `input.mdx` - Map of file names to MDX content strings
- `input.components` - Optional map of component names to definitions

**Returns:** `RenderResult` containing:
- `total`: Total number of files processed
- `succeeded`: Number of files that rendered successfully
- `failed`: Number of files that failed to render
- `errors`: Array of error objects with `file` and `message` properties
- `files`: Map of file names to render outcomes

**Throws:** `Error` if the request is invalid or an internal error occurs

## Requirements

- Node.js >= 18
- **Server-side only** - This package uses native Node.js addons and cannot be bundled for browser use

## Important: Server-Side Only

This package contains native `.node` binaries and is designed for **server-side Node.js environments only**. It cannot be used in:

- Browser bundles (Webpack, Vite, esbuild, Rollup, etc.)
- Edge runtimes (Cloudflare Workers, Vercel Edge)
- Browser-based environments

### Using with Vite/Next.js

If you're using Vite or Next.js, ensure the package is only imported in server-side code:

```javascript
// ✅ Correct: Server-side only (API routes, SSR, etc.)
// pages/api/render.js or app/api/render/route.js
import { Renderer } from '@dinja/core';

export async function POST(request) {
  const renderer = new Renderer();
  // ... render MDX server-side
}

// ❌ Wrong: Client-side code (will fail to bundle)
// components/MyComponent.jsx
import { Renderer } from '@dinja/core'; // Error: No loader for .node files
```

For Vite projects, you may need to exclude it from optimization:

```javascript
// vite.config.js
export default {
  optimizeDeps: {
    exclude: ['@dinja/core']
  },
  ssr: {
    noExternal: ['@dinja/core']
  }
}
```

## Platform Support

Pre-built binaries are provided for:
- Windows (x64)
- macOS (x64, ARM64)
- Linux (x64 glibc)

## License

BSD 3-Clause. See `LICENSE`.

## Links

- [GitHub](https://github.com/hlop3z/dinja)
- [Documentation](https://hlop3z.github.io/dinja)
- [npm](https://www.npmjs.com/package/@dinja/core)
