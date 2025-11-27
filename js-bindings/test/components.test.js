const { test, describe } = require('node:test');
const assert = require('node:assert');
const { Renderer } = require('..');

describe('Components', () => {
  test('should render with custom components', () => {
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
export function Button({ children }) {
  return <button class="btn">{children}</button>;
}
          `
        }
      }
    });

    assert.ok(result.files['app.mdx'].success);
    assert.ok(result.files['app.mdx'].output.includes('button'));
    assert.ok(result.files['app.mdx'].output.includes('Click me'));
  });

  test('should render component with props', () => {
    const renderer = new Renderer();

    const result = renderer.render({
      settings: { output: 'html', minify: false },
      mdx: {
        'app.mdx': `
import { Card } from './card';

<Card title="My Title" description="My Description" />
        `
      },
      components: {
        Card: {
          name: 'Card',
          code: `
export function Card({ title, description }) {
  return (
    <div class="card">
      <h2>{title}</h2>
      <p>{description}</p>
    </div>
  );
}
          `,
          args: {
            title: { type: 'string', required: true },
            description: { type: 'string', required: true }
          }
        }
      }
    });

    assert.ok(result.files['app.mdx'].success);
    assert.ok(result.files['app.mdx'].output.includes('My Title'));
    assert.ok(result.files['app.mdx'].output.includes('My Description'));
  });

  test('should render multiple components', () => {
    const renderer = new Renderer();

    const result = renderer.render({
      settings: { output: 'html', minify: false },
      mdx: {
        'app.mdx': `
import { Button } from './button';
import { Card } from './card';

<Card title="Welcome">
  <Button>Get Started</Button>
</Card>
        `
      },
      components: {
        Button: {
          code: `
export function Button({ children }) {
  return <button>{children}</button>;
}
          `
        },
        Card: {
          code: `
export function Card({ title, children }) {
  return (
    <div class="card">
      <h3>{title}</h3>
      {children}
    </div>
  );
}
          `
        }
      }
    });

    assert.ok(result.files['app.mdx'].success);
    assert.ok(result.files['app.mdx'].output.includes('Welcome'));
    assert.ok(result.files['app.mdx'].output.includes('Get Started'));
  });

  test('should include component documentation', () => {
    const renderer = new Renderer();

    const result = renderer.render({
      settings: { output: 'html', minify: false },
      mdx: {
        'app.mdx': `
import { Alert } from './alert';

<Alert type="info">Important message</Alert>
        `
      },
      components: {
        Alert: {
          name: 'Alert',
          code: `
export function Alert({ type, children }) {
  return <div class={\`alert alert-\${type}\`}>{children}</div>;
}
          `,
          docs: 'Displays an alert message with a specific type',
          args: {
            type: {
              type: 'string',
              required: true,
              description: 'The alert type (info, warning, error, success)'
            },
            children: {
              type: 'node',
              required: true,
              description: 'The alert content'
            }
          }
        }
      }
    });

    assert.ok(result.files['app.mdx'].success);
    assert.ok(result.files['app.mdx'].output.includes('Important message'));
  });
});
