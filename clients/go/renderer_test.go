package dinja_test

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	dinja "github.com/piny4man/dinja-go"
)

func TestNew(t *testing.T) {
	t.Run("default options", func(t *testing.T) {
		r := dinja.New()
		if r == nil {
			t.Fatal("expected non-nil renderer")
		}
	})

	t.Run("with custom options", func(t *testing.T) {
		client := &http.Client{Timeout: 10 * time.Second}
		r := dinja.New(
			dinja.WithBaseURL("http://custom:9000"),
			dinja.WithTimeout(60*time.Second),
			dinja.WithHTTPClient(client),
		)
		if r == nil {
			t.Fatal("expected non-nil renderer")
		}
	})
}

func TestHealth(t *testing.T) {
	t.Run("healthy service", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if r.URL.Path != "/health" {
				t.Errorf("expected path /health, got %s", r.URL.Path)
			}
			w.WriteHeader(http.StatusOK)
		}))
		defer server.Close()

		r := dinja.New(dinja.WithBaseURL(server.URL))
		healthy, err := r.Health(context.Background())
		if err != nil {
			t.Fatalf("unexpected error: %v", err)
		}
		if !healthy {
			t.Error("expected healthy to be true")
		}
	})

	t.Run("unhealthy service", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.WriteHeader(http.StatusServiceUnavailable)
		}))
		defer server.Close()

		r := dinja.New(dinja.WithBaseURL(server.URL))
		healthy, err := r.Health(context.Background())
		if err != nil {
			t.Fatalf("unexpected error: %v", err)
		}
		if healthy {
			t.Error("expected healthy to be false")
		}
	})
}

func TestRenderHTML(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/render/html" {
			t.Errorf("expected path /render/html, got %s", r.URL.Path)
		}
		if r.Method != http.MethodPost {
			t.Errorf("expected POST method, got %s", r.Method)
		}
		if ct := r.Header.Get("Content-Type"); ct != "application/json" {
			t.Errorf("expected Content-Type application/json, got %s", ct)
		}

		var input dinja.Input
		if err := json.NewDecoder(r.Body).Decode(&input); err != nil {
			t.Fatalf("failed to decode request: %v", err)
		}

		if input.Views["test.mdx"] != "# Hello World" {
			t.Errorf("unexpected view content: %s", input.Views["test.mdx"])
		}

		response := dinja.Result{
			Total:     1,
			Succeeded: 1,
			Failed:    0,
			Files: map[string]dinja.FileResult{
				"test.mdx": {
					Success: true,
					Result: &dinja.FileOutput{
						Metadata: map[string]any{},
						Output:   "<h1>Hello World</h1>",
					},
				},
			},
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(response)
	}))
	defer server.Close()

	r := dinja.New(dinja.WithBaseURL(server.URL))
	result, err := r.HTML(context.Background(), dinja.Input{
		Views: map[string]string{"test.mdx": "# Hello World"},
	})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if result.Total != 1 {
		t.Errorf("expected total 1, got %d", result.Total)
	}
	if result.Succeeded != 1 {
		t.Errorf("expected succeeded 1, got %d", result.Succeeded)
	}
	if !result.IsAllSuccess() {
		t.Error("expected all success")
	}
	if output := result.GetOutput("test.mdx"); output != "<h1>Hello World</h1>" {
		t.Errorf("unexpected output: %s", output)
	}
}

func TestRenderWithComponents(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		var input dinja.Input
		if err := json.NewDecoder(r.Body).Decode(&input); err != nil {
			t.Fatalf("failed to decode request: %v", err)
		}

		if comp, ok := input.Components["Button"]; !ok {
			t.Error("expected Button component")
		} else if comp.Code == "" {
			t.Error("expected component code")
		}

		response := dinja.Result{
			Total:     1,
			Succeeded: 1,
			Failed:    0,
			Files: map[string]dinja.FileResult{
				"test.mdx": {
					Success: true,
					Result: &dinja.FileOutput{
						Metadata: map[string]any{},
						Output:   "<button>Click me</button>",
					},
				},
			},
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(response)
	}))
	defer server.Close()

	r := dinja.New(dinja.WithBaseURL(server.URL))
	result, err := r.HTML(context.Background(), dinja.Input{
		Views: map[string]string{"test.mdx": "<Button>Click me</Button>"},
		Components: map[string]dinja.Component{
			"Button": {
				Code: "export default function Button({children}) { return <button>{children}</button>; }",
				Name: "Button",
			},
		},
	})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if !result.IsAllSuccess() {
		t.Error("expected all success")
	}
}

func TestRenderBatch(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		var input dinja.Input
		if err := json.NewDecoder(r.Body).Decode(&input); err != nil {
			t.Fatalf("failed to decode request: %v", err)
		}

		if len(input.Views) != 3 {
			t.Errorf("expected 3 views, got %d", len(input.Views))
		}

		response := dinja.Result{
			Total:     3,
			Succeeded: 3,
			Failed:    0,
			Files: map[string]dinja.FileResult{
				"page1.mdx": {Success: true, Result: &dinja.FileOutput{Output: "<h1>Page 1</h1>"}},
				"page2.mdx": {Success: true, Result: &dinja.FileOutput{Output: "<h1>Page 2</h1>"}},
				"page3.mdx": {Success: true, Result: &dinja.FileOutput{Output: "<h1>Page 3</h1>"}},
			},
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(response)
	}))
	defer server.Close()

	r := dinja.New(dinja.WithBaseURL(server.URL))
	result, err := r.HTML(context.Background(), dinja.Input{
		Views: map[string]string{
			"page1.mdx": "# Page 1",
			"page2.mdx": "# Page 2",
			"page3.mdx": "# Page 3",
		},
	})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if result.Total != 3 {
		t.Errorf("expected total 3, got %d", result.Total)
	}
	if !result.IsAllSuccess() {
		t.Error("expected all success")
	}
}

func TestRenderError(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusBadRequest)
		json.NewEncoder(w).Encode(map[string]string{"error": "invalid MDX syntax"})
	}))
	defer server.Close()

	r := dinja.New(dinja.WithBaseURL(server.URL))
	_, err := r.HTML(context.Background(), dinja.Input{
		Views: map[string]string{"test.mdx": "invalid {{{"},
	})
	if err == nil {
		t.Fatal("expected error")
	}
}

func TestRenderPartialFailure(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		response := dinja.Result{
			Total:     2,
			Succeeded: 1,
			Failed:    1,
			Files: map[string]dinja.FileResult{
				"good.mdx": {Success: true, Result: &dinja.FileOutput{Output: "<p>OK</p>"}},
				"bad.mdx":  {Success: false, Error: "syntax error"},
			},
			Errors: []dinja.ErrorInfo{{File: "bad.mdx", Message: "syntax error"}},
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(response)
	}))
	defer server.Close()

	r := dinja.New(dinja.WithBaseURL(server.URL))
	result, err := r.HTML(context.Background(), dinja.Input{
		Views: map[string]string{
			"good.mdx": "OK",
			"bad.mdx":  "bad {{{",
		},
	})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if result.IsAllSuccess() {
		t.Error("expected partial failure")
	}
	if result.Succeeded != 1 {
		t.Errorf("expected 1 success, got %d", result.Succeeded)
	}
	if result.Failed != 1 {
		t.Errorf("expected 1 failure, got %d", result.Failed)
	}
	if errMsg := result.GetError("bad.mdx"); errMsg != "syntax error" {
		t.Errorf("unexpected error message: %s", errMsg)
	}
}

func TestRenderOptions(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		var input dinja.Input
		if err := json.NewDecoder(r.Body).Decode(&input); err != nil {
			t.Fatalf("failed to decode request: %v", err)
		}

		if input.Utils != "export default { greeting: 'Hello' }" {
			t.Errorf("unexpected utils: %s", input.Utils)
		}
		if input.Minify == nil || *input.Minify != false {
			t.Error("expected minify to be false")
		}
		if len(input.Directives) != 2 || input.Directives[0] != "v-" {
			t.Errorf("unexpected directives: %v", input.Directives)
		}

		response := dinja.Result{
			Total:     1,
			Succeeded: 1,
			Files: map[string]dinja.FileResult{
				"test.mdx": {Success: true, Result: &dinja.FileOutput{Output: "<p>test</p>"}},
			},
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(response)
	}))
	defer server.Close()

	r := dinja.New(dinja.WithBaseURL(server.URL))
	opts := dinja.NewRenderOptions().
		WithView("test.mdx", "# Test").
		WithUtils("export default { greeting: 'Hello' }").
		WithMinify(false).
		WithDirectives("v-", "@")

	result, err := r.HTML(context.Background(), opts.Build())
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !result.IsAllSuccess() {
		t.Error("expected success")
	}
}

func TestAllOutputFormats(t *testing.T) {
	tests := []struct {
		name   string
		method func(*dinja.Renderer, context.Context, dinja.Input) (*dinja.Result, error)
		path   string
	}{
		{"HTML", (*dinja.Renderer).HTML, "/render/html"},
		{"JavaScript", (*dinja.Renderer).JavaScript, "/render/javascript"},
		{"Schema", (*dinja.Renderer).Schema, "/render/schema"},
		{"JSON", (*dinja.Renderer).JSON, "/render/json"},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
				if r.URL.Path != tt.path {
					t.Errorf("expected path %s, got %s", tt.path, r.URL.Path)
				}
				response := dinja.Result{
					Total:     1,
					Succeeded: 1,
					Files: map[string]dinja.FileResult{
						"test.mdx": {Success: true, Result: &dinja.FileOutput{Output: "output"}},
					},
				}
				w.Header().Set("Content-Type", "application/json")
				json.NewEncoder(w).Encode(response)
			}))
			defer server.Close()

			r := dinja.New(dinja.WithBaseURL(server.URL))
			result, err := tt.method(r, context.Background(), dinja.Input{
				Views: map[string]string{"test.mdx": "# Test"},
			})
			if err != nil {
				t.Fatalf("unexpected error: %v", err)
			}
			if !result.IsAllSuccess() {
				t.Error("expected success")
			}
		})
	}
}

func TestRenderGeneric(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/render/schema" {
			t.Errorf("expected path /render/schema, got %s", r.URL.Path)
		}
		response := dinja.Result{
			Total:     1,
			Succeeded: 1,
			Files: map[string]dinja.FileResult{
				"test.mdx": {Success: true, Result: &dinja.FileOutput{Output: "{}"}},
			},
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(response)
	}))
	defer server.Close()

	r := dinja.New(dinja.WithBaseURL(server.URL))
	result, err := r.Render(context.Background(), dinja.OutputSchema, dinja.Input{
		Views: map[string]string{"test.mdx": "# Test"},
	})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !result.IsAllSuccess() {
		t.Error("expected success")
	}
}

func TestContextCancellation(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		time.Sleep(100 * time.Millisecond)
		w.WriteHeader(http.StatusOK)
	}))
	defer server.Close()

	r := dinja.New(dinja.WithBaseURL(server.URL))
	ctx, cancel := context.WithCancel(context.Background())
	cancel() // Cancel immediately

	_, err := r.HTML(ctx, dinja.Input{
		Views: map[string]string{"test.mdx": "# Test"},
	})
	if err == nil {
		t.Fatal("expected error due to cancelled context")
	}
}
