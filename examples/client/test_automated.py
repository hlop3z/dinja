"""Automated test that starts the server, runs tests, and shuts it down.

This script:
1. Starts the HTTP server in the background
2. Waits for it to be ready
3. Runs tests for all output formats
4. Shuts down the server
"""

from __future__ import annotations

import json
import subprocess
import sys
import time
from pathlib import Path

try:
    import requests
except ImportError:
    print("Error: 'requests' library is required")
    print("Install it with: uv add requests")
    sys.exit(1)


def find_server_process():
    """Find the running server process."""
    try:
        # On Windows, use tasklist
        if sys.platform == "win32":
            result = subprocess.run(
                ["tasklist", "/FI", "IMAGENAME eq dinja-core.exe"],
                capture_output=True,
                text=True,
            )
            if "dinja-core.exe" in result.stdout:
                return True
        else:
            # On Unix, use ps
            result = subprocess.run(
                ["ps", "aux"], capture_output=True, text=True, check=False
            )
            if "cargo run --features http" in result.stdout or "dinja-core" in result.stdout:
                return True
    except Exception:
        pass
    return False


def start_server(project_root: Path) -> subprocess.Popen | None:
    """Start the HTTP server in the background."""
    print("Starting HTTP server...")
    core_dir = project_root / "core"
    
    if not core_dir.exists():
        print(f"✗ Error: core directory not found at {core_dir}")
        return None
    
    # Start server in background
    try:
        if sys.platform == "win32":
            # Windows: use start to run in new window, or run directly
            process = subprocess.Popen(
                ["cargo", "run", "--features", "http"],
                cwd=str(core_dir),
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                creationflags=subprocess.CREATE_NEW_PROCESS_GROUP,
            )
        else:
            # Unix: run in background
            process = subprocess.Popen(
                ["cargo", "run", "--features", "http"],
                cwd=str(core_dir),
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                start_new_session=True,
            )
        
        print(f"✓ Server process started (PID: {process.pid})")
        return process
    except Exception as e:
        print(f"✗ Failed to start server: {e}")
        return None


def wait_for_server(server_url: str, max_wait: int = 30) -> bool:
    """Wait for the server to be ready."""
    print(f"Waiting for server at {server_url}...")
    start_time = time.time()
    
    while time.time() - start_time < max_wait:
        try:
            response = requests.get(server_url, timeout=2)
            if response.status_code in [200, 404, 405]:  # Any response means server is up
                print("✓ Server is ready!")
                return True
        except requests.exceptions.ConnectionError:
            pass
        except Exception:
            pass
        
        time.sleep(0.5)
        print(".", end="", flush=True)
    
    print("\n✗ Server did not become ready in time")
    return False


def stop_server(process: subprocess.Popen | None):
    """Stop the HTTP server process."""
    if process is None:
        return
    
    print("\nStopping server...")
    try:
        if sys.platform == "win32":
            # Windows: terminate the process group
            subprocess.run(
                ["taskkill", "/F", "/T", "/PID", str(process.pid)],
                capture_output=True,
                check=False,
            )
        else:
            # Unix: send SIGTERM to process group
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
        
        print("✓ Server stopped")
    except Exception as e:
        print(f"⚠ Warning: Error stopping server: {e}")


def test_output_format(
    server_url: str, output_format: str, components: dict, mdx_content: str
) -> bool:
    """Test a specific output format."""
    render_endpoint = f"{server_url}/render"
    
    payload = {
        "settings": {
            "output": output_format,
            "minify": False,
        },
        "mdx": {"test.mdx": mdx_content},
        "components": components,
    }
    
    try:
        response = requests.post(
            render_endpoint,
            json=payload,
            headers={"Content-Type": "application/json"},
            timeout=30,
        )
        
        if response.status_code != 200:
            print(f"  ✗ {output_format.upper()}: HTTP {response.status_code}")
            try:
                error_data = response.json()
                print(f"     Error: {error_data.get('error', 'Unknown')}")
            except:
                print(f"     Response: {response.text[:200]}")
            return False
        
        result = response.json()
        
        if result["succeeded"] != 1 or result["failed"] != 0:
            print(f"  ✗ {output_format.upper()}: {result['succeeded']} succeeded, {result['failed']} failed")
            return False
        
        file_result = result["files"]["test.mdx"]
        if file_result["status"] != "success":
            print(f"  ✗ {output_format.upper()}: {file_result.get('error', 'Unknown error')}")
            return False
        
        output = file_result["result"].get("output", "")
        if not output:
            print(f"  ✗ {output_format.upper()}: No output generated")
            return False
        
        print(f"  ✓ {output_format.upper()}: Success (output length: {len(output)})")
        return True
        
    except Exception as e:
        print(f"  ✗ {output_format.upper()}: {e}")
        return False


def main() -> None:
    """Run automated tests."""
    print("=" * 60)
    print("Automated Dinja HTTP Server Test")
    print("=" * 60)
    print()
    
    # Get project root (two levels up from examples/client)
    script_dir = Path(__file__).parent
    project_root = script_dir.parent.parent
    
    server_url = "http://127.0.0.1:8080"
    server_process = None
    
    try:
        # Start server
        server_process = start_server(project_root)
        if server_process is None:
            print("✗ Failed to start server")
            sys.exit(1)
        
        # Wait for server to be ready
        if not wait_for_server(server_url):
            print("✗ Server did not start properly")
            sys.exit(1)
        
        # Give server a moment to fully initialize
        time.sleep(1)
        
        # Define test components and MDX
        components = {
            "Button": {
                "name": "Button",
                "code": "function Component(props) { return <button class={props.class || 'btn'}>{props.children}</button>; }",
                "docs": "A button component",
                "args": None,
            },
            "Card": {
                "name": "Card",
                "code": "function Component(props) { return <div class='card'><h3>{props.title}</h3><div class='card-content'>{props.children}</div></div>; }",
                "docs": "A card component",
                "args": None,
            },
            "Greeting": {
                "name": "Greeting",
                "code": "function Component(props) { return <div>Hello, <strong>{props.name}</strong>!</div>; }",
                "docs": "A greeting component",
                "args": None,
            },
        }
        
        mdx_content = """---
title: Test Page
author: Test Author
---

# {context('title')}

Welcome!

<Greeting name="World" />

<Card title="Test Card">
<p>This is a test card.</p>
<Button>Click Me</Button>
</Card>
"""
        
        # Test all output formats
        print("\nTesting output formats:")
        print("-" * 60)
        
        formats = ["html", "javascript", "schema", "json"]
        results = {}
        
        for fmt in formats:
            results[fmt] = test_output_format(server_url, fmt, components, mdx_content)
        
        # Summary
        print("\n" + "-" * 60)
        print("Test Summary:")
        passed = sum(1 for v in results.values() if v)
        total = len(results)
        
        for fmt, success in results.items():
            status = "✓ PASS" if success else "✗ FAIL"
            print(f"  {fmt.upper()}: {status}")
        
        print(f"\nTotal: {passed}/{total} passed")
        
        if passed == total:
            print("\n✓ All tests passed!")
            sys.exit(0)
        else:
            print("\n✗ Some tests failed")
            sys.exit(1)
            
    except KeyboardInterrupt:
        print("\n\nInterrupted by user")
        sys.exit(1)
    except Exception as e:
        print(f"\n✗ Unexpected error: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
    finally:
        # Always stop the server
        stop_server(server_process)


if __name__ == "__main__":
    main()

