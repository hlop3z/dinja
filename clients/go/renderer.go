package dinja

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"
)

const (
	defaultBaseURL = "http://localhost:8080"
	defaultTimeout = 30 * time.Second
)

// Option configures the Renderer.
type Option func(*Renderer)

// WithBaseURL sets the base URL of the Dinja service.
func WithBaseURL(url string) Option {
	return func(r *Renderer) {
		r.baseURL = url
	}
}

// WithTimeout sets the HTTP request timeout.
func WithTimeout(timeout time.Duration) Option {
	return func(r *Renderer) {
		r.timeout = timeout
	}
}

// WithHTTPClient sets a custom HTTP client.
func WithHTTPClient(client *http.Client) Option {
	return func(r *Renderer) {
		r.client = client
	}
}

// Renderer is an HTTP client for the Dinja MDX rendering service.
type Renderer struct {
	baseURL string
	timeout time.Duration
	client  *http.Client
}

// New creates a new Renderer with the given options.
func New(opts ...Option) *Renderer {
	r := &Renderer{
		baseURL: defaultBaseURL,
		timeout: defaultTimeout,
	}

	for _, opt := range opts {
		opt(r)
	}

	if r.client == nil {
		r.client = &http.Client{
			Timeout: r.timeout,
		}
	}

	return r
}

// Health checks if the Dinja service is healthy.
func (r *Renderer) Health(ctx context.Context) (bool, error) {
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, r.baseURL+"/health", nil)
	if err != nil {
		return false, fmt.Errorf("creating request: %w", err)
	}

	resp, err := r.client.Do(req)
	if err != nil {
		return false, fmt.Errorf("health check failed: %w", err)
	}
	defer resp.Body.Close()

	return resp.StatusCode == http.StatusOK, nil
}

// Render renders MDX content with the specified output format.
func (r *Renderer) Render(ctx context.Context, output Output, input Input) (*Result, error) {
	return r.doRender(ctx, string(output), input)
}

// HTML renders MDX content to HTML.
func (r *Renderer) HTML(ctx context.Context, input Input) (*Result, error) {
	return r.doRender(ctx, "html", input)
}

// JavaScript renders MDX content to JavaScript.
func (r *Renderer) JavaScript(ctx context.Context, input Input) (*Result, error) {
	return r.doRender(ctx, "javascript", input)
}

// Schema extracts the component schema from MDX content.
func (r *Renderer) Schema(ctx context.Context, input Input) (*Result, error) {
	return r.doRender(ctx, "schema", input)
}

// JSON renders MDX content to a JSON tree representation.
func (r *Renderer) JSON(ctx context.Context, input Input) (*Result, error) {
	return r.doRender(ctx, "json", input)
}

func (r *Renderer) doRender(ctx context.Context, output string, input Input) (*Result, error) {
	// Set default minify to true if not specified
	if input.Minify == nil {
		minify := true
		input.Minify = &minify
	}

	body, err := json.Marshal(input)
	if err != nil {
		return nil, fmt.Errorf("marshaling request: %w", err)
	}

	url := fmt.Sprintf("%s/render/%s", r.baseURL, output)
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, url, bytes.NewReader(body))
	if err != nil {
		return nil, fmt.Errorf("creating request: %w", err)
	}
	req.Header.Set("Content-Type", "application/json")

	resp, err := r.client.Do(req)
	if err != nil {
		return nil, fmt.Errorf("request failed: %w", err)
	}
	defer resp.Body.Close()

	respBody, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("reading response: %w", err)
	}

	if resp.StatusCode != http.StatusOK {
		var errResp struct {
			Error string `json:"error"`
		}
		if json.Unmarshal(respBody, &errResp) == nil && errResp.Error != "" {
			return nil, fmt.Errorf("render failed: %s", errResp.Error)
		}
		return nil, fmt.Errorf("render failed with status %d: %s", resp.StatusCode, string(respBody))
	}

	var result Result
	if err := json.Unmarshal(respBody, &result); err != nil {
		return nil, fmt.Errorf("unmarshaling response: %w", err)
	}

	return &result, nil
}

// RenderOptions provides a builder pattern for render requests.
type RenderOptions struct {
	views      map[string]string
	components map[string]Component
	utils      string
	minify     *bool
	directives []string
}

// NewRenderOptions creates a new RenderOptions builder.
func NewRenderOptions() *RenderOptions {
	return &RenderOptions{
		views:      make(map[string]string),
		components: make(map[string]Component),
	}
}

// WithView adds a single MDX view.
func (o *RenderOptions) WithView(filename, content string) *RenderOptions {
	o.views[filename] = content
	return o
}

// WithViews adds multiple MDX views.
func (o *RenderOptions) WithViews(views map[string]string) *RenderOptions {
	for k, v := range views {
		o.views[k] = v
	}
	return o
}

// WithComponent adds a component with full definition.
func (o *RenderOptions) WithComponent(name string, component Component) *RenderOptions {
	o.components[name] = component
	return o
}

// WithComponentCode adds a component with just the code.
func (o *RenderOptions) WithComponentCode(name, code string) *RenderOptions {
	o.components[name] = Component{Code: code, Name: name}
	return o
}

// WithUtils sets the global utilities code.
func (o *RenderOptions) WithUtils(utils string) *RenderOptions {
	o.utils = utils
	return o
}

// WithMinify sets whether to minify the output.
func (o *RenderOptions) WithMinify(minify bool) *RenderOptions {
	o.minify = &minify
	return o
}

// WithDirectives sets directive prefixes for schema extraction.
func (o *RenderOptions) WithDirectives(directives ...string) *RenderOptions {
	o.directives = directives
	return o
}

// Build creates the Input from the options.
func (o *RenderOptions) Build() Input {
	input := Input{
		Views:      o.views,
		Utils:      o.utils,
		Minify:     o.minify,
		Directives: o.directives,
	}
	if len(o.components) > 0 {
		input.Components = o.components
	}
	return input
}
