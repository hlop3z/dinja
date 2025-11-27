import { useState, useEffect } from "preact/hooks";

// Component definitions (sent to server for MDX rendering)
const components = {
  Alert: {
    name: "Alert",
    code: `export default function Component({ type, children }) {
  const colors = {
    info: { bg: '#1e3a5f', border: '#3b82f6' },
    warning: { bg: '#5f4b1e', border: '#f59e0b' },
    error: { bg: '#5f1e1e', border: '#ef4444' }
  };
  const style = colors[type] || colors.info;
  return (
    <div style={{
      background: style.bg,
      borderLeft: '4px solid ' + style.border,
      padding: '1rem',
      borderRadius: '4px',
      margin: '1rem 0'
    }}>
      {children}
    </div>
  );
}`,
  },
};

const defaultMdx = `# Hello **Dinja**!

This is rendered server-side using Rust.

<Alert type="info">
  This component is defined in JavaScript!
</Alert>

## Features
- Fast rendering with V8
- Custom components
- YAML frontmatter support`;

export function App() {
  const [mdx, setMdx] = useState(defaultMdx);
  const [html, setHtml] = useState("");
  const [rawHtml, setRawHtml] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  const renderMdx = async () => {
    setLoading(true);
    setError(null);

    try {
      const response = await fetch("/api/render", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          mdx: { "page.mdx": mdx },
          components,
          settings: { output: "html", minify: false },
        }),
      });

      const result = await response.json();

      if (result.files?.["page.mdx"]) {
        const file = result.files["page.mdx"];
        if (file.status === "success") {
          setHtml(file.result.output);
          setRawHtml(file.result.output);
        } else {
          setError(file.error);
        }
      } else if (result.error) {
        setError(result.error);
      }
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    renderMdx();
  }, []);

  return (
    <div class="container">
      <header>
        <h1>🦀 Dinja + Vite + Preact</h1>
        <p>Server-side MDX rendering with @dinja/core</p>
      </header>

      <div class="panels">
        <div class="panel">
          <h2>MDX Input</h2>
          <textarea
            value={mdx}
            onInput={(e) => setMdx(e.target.value)}
            placeholder="Enter MDX content..."
          />
          <button onClick={renderMdx} disabled={loading}>
            {loading ? "Rendering..." : "Render MDX"}
          </button>
        </div>

        <div class="panel">
          <h2>HTML Output</h2>
          <div class="output">
            {error ? (
              <p class="error">Error: {error}</p>
            ) : (
              <div dangerouslySetInnerHTML={{ __html: html }} />
            )}
          </div>
          {rawHtml && <pre class="raw">{rawHtml}</pre>}
        </div>
      </div>
    </div>
  );
}
