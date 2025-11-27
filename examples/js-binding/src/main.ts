import { Renderer } from "@dinja/core";

// Create a renderer instance (engine loads once)
const renderer = new Renderer();

// Render MDX content
const result = renderer.render({
  settings: {
    output: "html",
    minify: false,
  },
  mdx: {
    "example.mdx": "# Hello **dinja**",
  },
});

console.log(result);
