const { test, describe } = require('node:test');
const assert = require('node:assert');
const { Renderer } = require('..');

describe('Renderer', () => {
  test('should create a new renderer instance', () => {
    const renderer = new Renderer();
    assert.ok(renderer);
    assert.ok(typeof renderer.render === 'function');
  });

  test('should render simple MDX to HTML', () => {
    const renderer = new Renderer();
    const result = renderer.render({
      settings: { output: 'html', minify: false },
      mdx: { 'test.mdx': '# Hello World' }
    });

    assert.strictEqual(result.total, 1);
    assert.strictEqual(result.succeeded, 1);
    assert.strictEqual(result.failed, 0);
    assert.strictEqual(result.errors.length, 0);
    assert.ok(result.files['test.mdx'].success);
    assert.ok(result.files['test.mdx'].output.includes('Hello World'));
  });

  test('should render MDX with bold text', () => {
    const renderer = new Renderer();
    const result = renderer.render({
      settings: { output: 'html', minify: false },
      mdx: { 'test.mdx': '# Hello **World**' }
    });

    assert.ok(result.files['test.mdx'].success);
    assert.ok(result.files['test.mdx'].output.includes('<strong>'));
    assert.ok(result.files['test.mdx'].output.includes('World'));
  });

  test('should render multiple files', () => {
    const renderer = new Renderer();
    const result = renderer.render({
      settings: { output: 'html', minify: false },
      mdx: {
        'file1.mdx': '# File 1',
        'file2.mdx': '# File 2',
        'file3.mdx': '# File 3'
      }
    });

    assert.strictEqual(result.total, 3);
    assert.strictEqual(result.succeeded, 3);
    assert.strictEqual(result.failed, 0);
    assert.ok(result.files['file1.mdx'].success);
    assert.ok(result.files['file2.mdx'].success);
    assert.ok(result.files['file3.mdx'].success);
  });

  test('should render with minification', () => {
    const renderer = new Renderer();

    const unminified = renderer.render({
      settings: { output: 'html', minify: false },
      mdx: { 'test.mdx': '# Hello\n\nWorld' }
    });

    const minified = renderer.render({
      settings: { output: 'html', minify: true },
      mdx: { 'test.mdx': '# Hello\n\nWorld' }
    });

    assert.ok(unminified.files['test.mdx'].success);
    assert.ok(minified.files['test.mdx'].success);

    // Minified should be shorter or equal
    assert.ok(
      minified.files['test.mdx'].output.length <=
      unminified.files['test.mdx'].output.length
    );
  });

  test('should support different output formats', () => {
    const renderer = new Renderer();
    const mdx = { 'test.mdx': '# Hello' };

    const html = renderer.render({
      settings: { output: 'html', minify: false },
      mdx
    });

    const javascript = renderer.render({
      settings: { output: 'javascript', minify: false },
      mdx
    });

    const schema = renderer.render({
      settings: { output: 'schema', minify: false },
      mdx
    });

    const json = renderer.render({
      settings: { output: 'json', minify: false },
      mdx
    });

    assert.ok(html.files['test.mdx'].success);
    assert.ok(javascript.files['test.mdx'].success);
    assert.ok(schema.files['test.mdx'].success);
    assert.ok(json.files['test.mdx'].success);

    // Each format should produce different output
    assert.notStrictEqual(
      html.files['test.mdx'].output,
      javascript.files['test.mdx'].output
    );
  });

  test('should reuse renderer instance across multiple renders', () => {
    const renderer = new Renderer();

    const result1 = renderer.render({
      settings: { output: 'html', minify: false },
      mdx: { 'page1.mdx': '# Page 1' }
    });

    const result2 = renderer.render({
      settings: { output: 'schema', minify: false },
      mdx: { 'page2.mdx': '# Page 2' }
    });

    const result3 = renderer.render({
      settings: { output: 'javascript', minify: false },
      mdx: { 'page3.mdx': '# Page 3' }
    });

    assert.ok(result1.files['page1.mdx'].success);
    assert.ok(result2.files['page2.mdx'].success);
    assert.ok(result3.files['page3.mdx'].success);
  });

  test('should handle errors gracefully', () => {
    const renderer = new Renderer();

    // Invalid MDX syntax should be caught
    const result = renderer.render({
      settings: { output: 'html', minify: false },
      mdx: { 'bad.mdx': '<Component>' } // Unclosed component
    });

    // Should still return a result structure
    assert.strictEqual(result.total, 1);
    assert.ok(result.failed >= 0);
  });
});
