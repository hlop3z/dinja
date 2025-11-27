/**
 * Output format for rendered MDX
 */
export type OutputFormat = 'html' | 'javascript' | 'schema' | 'json';

/**
 * Render settings
 */
export interface RenderSettings {
  /** Output format */
  output: OutputFormat;
  /** Whether to minify the output */
  minify: boolean;
  /** Optional JavaScript snippet to inject as global utilities (must use `export default { ... }`) */
  utils?: string;
  /** Optional map of directive names to their string values for custom processing */
  directives?: Record<string, string>;
}

/**
 * Component definition
 */
export interface ComponentDefinition {
  /** Component name (optional) */
  name?: string;
  /** JavaScript code for the component */
  code: string;
  /** Documentation (optional) */
  docs?: string;
  /** Arguments schema (optional) */
  args?: Record<string, any>;
}

/**
 * Render request input
 */
export interface RenderInput {
  /** Render settings */
  settings: RenderSettings;
  /** Map of file names to MDX content */
  mdx: Record<string, string>;
  /** Optional map of component names to definitions */
  components?: Record<string, ComponentDefinition>;
}

/**
 * Render error details
 */
export interface RenderError {
  /** File that failed */
  file: string;
  /** Error message */
  message: string;
}

/**
 * Render result for a successful render
 */
export interface RenderSuccess {
  /** The status will be "success" */
  status: 'success';
  /** Render result */
  result: {
    /** Metadata extracted from frontmatter */
    metadata: Record<string, any>;
    /** Rendered output */
    output: string;
  };
}

/**
 * Render result for a failed render
 */
export interface RenderFailure {
  /** The status will be "error" */
  status: 'error';
  /** Error message */
  error: string;
}

/**
 * Render outcome for a single file
 */
export type RenderOutcome = RenderSuccess | RenderFailure;

/**
 * Batch render result
 */
export interface RenderResult {
  /** Total number of files */
  total: number;
  /** Number of successful renders */
  succeeded: number;
  /** Number of failed renders */
  failed: number;
  /** Map of file names to render outcomes */
  files: Record<string, RenderOutcome>;
}

/**
 * A reusable MDX renderer instance.
 *
 * This class maintains a single RenderService instance and reuses it across
 * multiple renders, which prevents v8 isolate issues and improves performance.
 *
 * @example
 * ```typescript
 * const { Renderer } = require('@dinja/core');
 *
 * // Create a renderer instance (loads engine once)
 * const renderer = new Renderer();
 *
 * // Reuse the same instance for multiple renders
 * const result1 = renderer.render({
 *   settings: { output: 'html', minify: false },
 *   mdx: { 'file1.mdx': '# Hello' }
 * });
 *
 * const result2 = renderer.render({
 *   settings: { output: 'schema', minify: false },
 *   mdx: { 'file2.mdx': '# World' }
 * });
 * ```
 */
export class Renderer {
  /**
   * Creates a new Renderer instance.
   *
   * The engine is loaded once during initialization and reused for all subsequent renders.
   */
  constructor();

  /**
   * Renders MDX content.
   *
   * @param input - Render input containing settings, MDX files, and optional components
   * @returns Render result with outcomes for all files
   * @throws {Error} If the request is invalid or an internal error occurs
   */
  render(input: RenderInput): RenderResult;
}
