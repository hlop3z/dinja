const { test, describe } = require('node:test');
const assert = require('node:assert');
const { Renderer } = require('..');

describe('Renderer', () => {
  // Single comprehensive test to avoid V8 isolate ordering issues
  test('should render MDX correctly', () => {
    const renderer = new Renderer();

    // Test basic rendering
    const basic = renderer.render({
      settings: { output: 'html', minify: false },
      mdx: { 'test.mdx': '# Hello World' }
    });

    assert.strictEqual(basic.total, 1);
    assert.strictEqual(basic.succeeded, 1);
    assert.strictEqual(basic.files['test.mdx'].status, 'success');
    assert.ok(basic.files['test.mdx'].result.output.includes('Hello World'));

    // Test multiple files in same render call
    const batch = renderer.render({
      settings: { output: 'html', minify: false },
      mdx: {
        'page1.mdx': '# Page 1',
        'page2.mdx': '# Page 2',
      }
    });

    assert.strictEqual(batch.total, 2);
    assert.strictEqual(batch.succeeded, 2);

    // Test different output formats
    const formats = ['html', 'javascript', 'schema', 'json'];
    for (const format of formats) {
      const result = renderer.render({
        settings: { output: format, minify: false },
        mdx: { 'test.mdx': '# Test' }
      });
      assert.strictEqual(result.files['test.mdx'].status, 'success', `Format ${format} should succeed`);
      assert.ok(result.files['test.mdx'].result.output, `Format ${format} should produce output`);
    }

    // Test with custom component
    const withComponent = renderer.render({
      settings: { output: 'html', minify: false },
      mdx: { 'app.mdx': '<Button>Click</Button>' },
      components: {
        Button: {
          name: 'Button',
          code: 'export default function Component({ children }) { return <button>{children}</button>; }'
        }
      }
    });

    assert.strictEqual(withComponent.files['app.mdx'].status, 'success');
    assert.ok(withComponent.files['app.mdx'].result.output.includes('button'));
    assert.ok(withComponent.files['app.mdx'].result.output.includes('Click'));
  });
});
