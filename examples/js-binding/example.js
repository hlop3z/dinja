import { Renderer } from "@dinja/core";

// Create a renderer instance (engine loads once)
const renderer = new Renderer();

// Basic rendering
console.log("=== Basic MDX Rendering ===\n");

const basic = renderer.render({
  settings: {
    output: "html",
    minify: false,
  },
  mdx: {
    "hello.mdx": "# Hello **dinja**\n\nThis is a paragraph.",
  },
});

console.log("Input: # Hello **dinja**");
console.log("Output:", basic.files["hello.mdx"].result.output);
console.log();

// With components
console.log("=== Custom Components ===\n");

const withComponents = renderer.render({
  settings: {
    output: "html",
    minify: false,
  },
  mdx: {
    "page.mdx": `import { Button } from './button';

<Button>Click me</Button>`,
  },
  components: {
    Button: {
      name: "Button",
      code: `export default function Component({ children }) {
  return <button class="btn">{children}</button>;
}`,
    },
  },
});

const compResult = withComponents.files["page.mdx"];
if (compResult.status === "success") {
  console.log("Output:", compResult.result.output);
} else {
  console.log("Error:", compResult.error);
}
console.log();

// With utils
console.log("=== Global Utils ===\n");

const withUtils = renderer.render({
  settings: {
    output: "html",
    minify: false,
    utils: "export default { siteName: 'My Site', version: '1.0.0' }",
  },
  mdx: {
    "about.mdx": "<SiteInfo />",
  },
  components: {
    SiteInfo: {
      name: "SiteInfo",
      code: `export default function Component() {
  return <p>Welcome to {utils.siteName} v{utils.version}</p>;
}`,
    },
  },
});

const utilsResult = withUtils.files["about.mdx"];
if (utilsResult.status === "success") {
  console.log("Output:", utilsResult.result.output);
} else {
  console.log("Error:", utilsResult.error);
}
console.log();

// Batch rendering
console.log("=== Batch Rendering ===\n");

const batch = renderer.render({
  settings: {
    output: "html",
    minify: false,
  },
  mdx: {
    "page1.mdx": "# Page 1",
    "page2.mdx": "# Page 2",
    "page3.mdx": "# Page 3",
  },
});

console.log(`Rendered ${batch.succeeded}/${batch.total} files`);
for (const [name, file] of Object.entries(batch.files)) {
  if (file.status === "success") {
    console.log(`  ${name}: ${file.result.output}`);
  }
}
