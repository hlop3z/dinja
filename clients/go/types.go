// Package dinja provides an HTTP client for the Dinja MDX rendering service.
package dinja

// Output represents the output format for rendering.
type Output string

const (
	OutputHTML       Output = "html"
	OutputJavaScript Output = "javascript"
	OutputSchema     Output = "schema"
	OutputJSON       Output = "json"
)

// Component represents a custom JSX component definition.
type Component struct {
	// Code is the JSX/TSX source code (required).
	Code string `json:"code"`
	// Name is the component name (optional).
	Name string `json:"name,omitempty"`
	// Docs is documentation for the component (optional).
	Docs string `json:"docs,omitempty"`
	// Args contains props type information as JSON (optional).
	Args any `json:"args,omitempty"`
}

// Input represents the request payload for rendering.
type Input struct {
	// Views maps filenames to MDX content (required).
	Views map[string]string `json:"mdx"`
	// Components maps component names to their definitions (optional).
	Components map[string]Component `json:"components,omitempty"`
	// Utils is global JavaScript utilities code (optional).
	Utils string `json:"utils,omitempty"`
	// Minify enables output minification (default: true).
	Minify *bool `json:"minify,omitempty"`
	// Directives lists directive prefixes to extract in schema mode (optional).
	Directives []string `json:"directives,omitempty"`
}

// FileResult represents the outcome of rendering a single file.
type FileResult struct {
	// Success indicates whether the file was rendered successfully.
	Success bool `json:"success"`
	// Result contains the rendered output and metadata on success.
	Result *FileOutput `json:"result,omitempty"`
	// Error contains the error message on failure.
	Error string `json:"error,omitempty"`
}

// FileOutput contains the rendered output and metadata.
type FileOutput struct {
	// Metadata contains frontmatter and other extracted data.
	Metadata map[string]any `json:"metadata"`
	// Output is the rendered content.
	Output string `json:"output,omitempty"`
}

// ErrorInfo represents an error for a specific file.
type ErrorInfo struct {
	File    string `json:"file"`
	Message string `json:"message"`
}

// Result represents the batch rendering response.
type Result struct {
	// Total is the number of files processed.
	Total int `json:"total"`
	// Succeeded is the number of successfully rendered files.
	Succeeded int `json:"succeeded"`
	// Failed is the number of files that failed to render.
	Failed int `json:"failed"`
	// Files maps filenames to their individual results.
	Files map[string]FileResult `json:"files"`
	// Errors contains error information for failed files.
	Errors []ErrorInfo `json:"errors,omitempty"`
}

// IsAllSuccess returns true if all files were rendered successfully.
func (r *Result) IsAllSuccess() bool {
	return r.Failed == 0 && r.Succeeded == r.Total
}

// GetOutput returns the rendered output for a specific file.
// Returns empty string if the file doesn't exist or failed.
func (r *Result) GetOutput(filename string) string {
	if file, ok := r.Files[filename]; ok && file.Success && file.Result != nil {
		return file.Result.Output
	}
	return ""
}

// GetMetadata returns the metadata for a specific file.
// Returns nil if the file doesn't exist or failed.
func (r *Result) GetMetadata(filename string) map[string]any {
	if file, ok := r.Files[filename]; ok && file.Success && file.Result != nil {
		return file.Result.Metadata
	}
	return nil
}

// GetError returns the error message for a specific file.
// Returns empty string if the file succeeded or doesn't exist.
func (r *Result) GetError(filename string) string {
	if file, ok := r.Files[filename]; ok && !file.Success {
		return file.Error
	}
	return ""
}
