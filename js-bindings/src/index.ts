/**
 * Dinja - MDX Rendering Client
 *
 * HTTP client for the Dinja MDX rendering service.
 * Connect to the Dinja service running via Docker:
 *   docker pull ghcr.io/hlop3z/dinja:latest
 *   docker run -p 8080:8080 ghcr.io/hlop3z/dinja:latest
 */

/** Output format options */
export type Output = "html" | "javascript" | "schema" | "json";

/** Component definition with code and metadata */
export interface Component {
  /** Component code (JSX/TSX) - required */
  code: string;
  /** Component name (optional, defaults to dict key) */
  name?: string;
  /** Component documentation (metadata) */
  docs?: string;
  /** Component arguments/props types (metadata) */
  args?: unknown;
}

/** Input structure for MDX rendering requests */
export interface Input {
  /** Map of file names to MDX content strings */
  mdx: Record<string, string>;
  /** Optional JavaScript snippet for global utilities (export default { ... }) */
  utils?: string;
  /** Optional map of component names to their definitions or code strings */
  components?: Record<string, Component | string>;
  /** Enable minification (default: true) */
  minify?: boolean;
  /** Optional list of directive prefixes for schema extraction */
  directives?: string[];
}

/** Result for a single rendered file */
export interface FileResult {
  /** Whether rendering succeeded */
  success: boolean;
  /** Render result (if successful) */
  result?: {
    /** Parsed YAML frontmatter metadata */
    metadata: Record<string, unknown>;
    /** Rendered output (HTML, JS, schema, or JSON depending on format) */
    output?: string;
  };
  /** Error message if rendering failed */
  error?: string;
}

/** Result of a batch render operation */
export interface Result {
  /** Total number of files processed */
  total: number;
  /** Number of files that rendered successfully */
  succeeded: number;
  /** Number of files that failed to render */
  failed: number;
  /** Dictionary mapping file names to FileResult */
  files: Record<string, FileResult>;
  /** List of error objects with file and message */
  errors: Array<{ file: string; message: string }>;
}

/** Renderer configuration options */
export interface RendererConfig {
  /** Base URL of the Dinja service (default: "http://localhost:8080") */
  baseUrl?: string;
  /** Request timeout in milliseconds (default: 30000) */
  timeout?: number;
}

/**
 * Build request data from input parameters
 */
function buildRequestData(input: Input): Record<string, unknown> {
  const result: Record<string, unknown> = {
    mdx: input.mdx,
    minify: input.minify ?? true,
  };

  if (input.utils !== undefined) {
    result.utils = input.utils;
  }

  if (input.directives !== undefined) {
    result.directives = input.directives;
  }

  if (input.components !== undefined) {
    const components: Record<string, Component> = {};
    for (const [name, comp] of Object.entries(input.components)) {
      if (typeof comp === "string") {
        components[name] = { code: comp, name };
      } else {
        components[name] = comp;
      }
    }
    result.components = components;
  }

  return result;
}

/**
 * HTTP client for the Dinja MDX rendering service.
 *
 * @example
 * ```typescript
 * import { Renderer } from '@dinja/core';
 *
 * // Connect to local Docker service
 * const renderer = new Renderer({ baseUrl: "http://localhost:8080" });
 *
 * // Render MDX to HTML
 * const result = await renderer.html({
 *   mdx: { "page.mdx": "# Hello World" },
 *   utils: "export default { greeting: 'Hello' }",
 * });
 *
 * console.log(result.files["page.mdx"].result?.output);
 * ```
 */
export class Renderer {
  private baseUrl: string;
  private timeout: number;

  /**
   * Create a new Renderer client.
   * @param config - Configuration options
   */
  constructor(config?: RendererConfig) {
    this.baseUrl = (config?.baseUrl ?? "http://localhost:8080").replace(
      /\/$/,
      ""
    );
    this.timeout = config?.timeout ?? 30000;
  }

  /**
   * Make HTTP POST request to the service.
   */
  private async request(
    endpoint: string,
    data: Record<string, unknown>
  ): Promise<Result> {
    const url = `${this.baseUrl}${endpoint}`;
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.timeout);

    try {
      const response = await fetch(url, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(data),
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      const responseData = await response.json();

      if (!response.ok) {
        throw new Error(
          (responseData as { error?: string }).error ??
            `HTTP ${response.status}`
        );
      }

      return responseData as Result;
    } catch (error) {
      clearTimeout(timeoutId);
      if (error instanceof Error && error.name === "AbortError") {
        throw new Error(`Request timeout after ${this.timeout}ms`);
      }
      throw error;
    }
  }

  /**
   * Render MDX to HTML.
   * @param input - Input data with mdx content, utils, and components
   * @returns Result with rendered HTML output
   */
  async html(input: Input): Promise<Result> {
    return this.request("/render/html", buildRequestData(input));
  }

  /**
   * Render MDX to JavaScript.
   * @param input - Input data with mdx content, utils, and components
   * @returns Result with JavaScript output
   */
  async javascript(input: Input): Promise<Result> {
    return this.request("/render/javascript", buildRequestData(input));
  }

  /**
   * Extract schema from MDX (component names).
   * @param input - Input data with mdx content
   * @returns Result with schema output
   */
  async schema(input: Input): Promise<Result> {
    return this.request("/render/schema", buildRequestData(input));
  }

  /**
   * Render MDX to JSON tree.
   * @param input - Input data with mdx content, utils, and components
   * @returns Result with JSON tree output
   */
  async json(input: Input): Promise<Result> {
    return this.request("/render/json", buildRequestData(input));
  }

  /**
   * Render MDX with specified output format.
   * @param output - Output format ("html", "javascript", "schema", "json")
   * @param input - Input data with mdx content, utils, and components
   * @returns Result with rendered output
   */
  async render(output: Output, input: Input): Promise<Result> {
    return this.request(`/render/${output}`, buildRequestData(input));
  }

  /**
   * Check if the service is healthy.
   * @returns True if service is healthy, false otherwise
   */
  async health(): Promise<boolean> {
    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), this.timeout);

      const response = await fetch(`${this.baseUrl}/health`, {
        method: "GET",
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      if (!response.ok) {
        return false;
      }

      const data = (await response.json()) as { status?: string };
      return data.status === "ok";
    } catch {
      return false;
    }
  }
}

// Helper functions for convenience
export function getOutput(result: Result, filename: string): string | undefined {
  return result.files[filename]?.result?.output;
}

export function getMetadata(
  result: Result,
  filename: string
): Record<string, unknown> {
  return result.files[filename]?.result?.metadata ?? {};
}

export function isAllSuccess(result: Result): boolean {
  return result.failed === 0 && result.succeeded === result.total;
}
