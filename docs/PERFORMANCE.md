# Performance & Capacity

## Stress Test Results

Tested on Windows with Rust release build.

### Component Capacity (Single View)

| Components | Time (ms) | Output (KB) | Status |
|------------|-----------|-------------|--------|
| 10         | 5         | 0.52        | OK     |
| 25         | 3         | 1.29        | OK     |
| 50         | 5         | 2.59        | OK     |
| 100        | 10        | 5.18        | OK     |
| 150        | 20        | 7.86        | OK     |
| 200        | 32        | 10.55       | OK     |
| 250        | 31        | 13.23       | OK     |
| 300        | 28        | 15.92       | OK     |
| 400        | 92        | 21.29       | OK     |
| 500        | 124       | 26.66       | OK     |

### Nesting Depth

| Depth | Time (ms) | Output (KB) | Status |
|-------|-----------|-------------|--------|
| 5     | ~1        | 0.2         | OK     |
| 10    | ~1        | 0.4         | OK     |
| 20    | ~1        | 0.8         | OK     |
| 30    | ~2        | 1.2         | OK     |
| 40    | ~2        | 1.6         | OK     |
| 50    | ~3        | 2.0         | OK     |

### Props per Component

| Props | Time (ms) | Output (KB) | Status |
|-------|-----------|-------------|--------|
| 5     | 2         | 0.12        | OK     |
| 10    | <1        | 0.22        | OK     |
| 20    | 1         | 0.41        | OK     |
| 30    | <1        | 0.61        | OK     |
| 50    | 1         | 1.00        | OK     |
| 75    | 1         | 1.49        | OK     |
| 100   | 1         | 1.97        | OK     |

### Content Size

| Input (KB) | Time (ms) | Output (KB) | Status |
|------------|-----------|-------------|--------|
| 1          | 2         | 0.94        | OK     |
| 5          | 3         | 4.57        | OK     |
| 10         | 5         | 9.11        | OK     |
| 50         | 34        | 45.44       | OK     |
| 100        | 89        | 90.85       | OK     |
| 250        | 286       | 227.08      | OK     |
| 500        | 1823      | 454.13      | OK     |
| 1000       | -         | -           | FAIL*  |

\* Stack overflow at ~600KB due to V8 engine call stack limit with deeply nested `engine.h()` calls.

## Summary

| Metric | Tested Limit | Status |
|--------|--------------|--------|
| Components per view | 500+ | Supported |
| Nesting depth | 50+ levels | Supported |
| Props per component | 100+ | Supported |
| Content size | ~500 KB | Supported |
| Content size | ~600 KB+ | V8 limit* |

## Known Limits

1. **V8 Call Stack**: Content with heavy inline formatting (bold, italic, code) in a single paragraph can hit V8's call stack limit at ~600KB. This is a JavaScript engine limitation, not a Dinja limitation.

2. **Resource Limits** (configurable):
   - `max_batch_size`: 1000 files per batch (default)
   - `max_mdx_content_size`: 10 MB per file (default)
   - `max_component_code_size`: 1 MB per component (default)
   - `max_cached_renderers`: 4 (default)
