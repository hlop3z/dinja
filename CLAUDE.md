# How to develop here

## Project map

Dinja is a safe, deterministic MDX (Markdown with JSX) renderer: a Rust core
(`core/`, crate `dinja-core`) exposed as an HTTP service, with thin HTTP clients in
`clients/py`, `clients/js`, and `clients/go`. Builds and tests go through
`./utils/build.sh` (see `/build`, `/test`, `/lint`, `/dev-setup`, `/release`).

Rendering pipeline:

```
MDX → YAML frontmatter (gray_matter) → Markdown→HTML (markdown crate)
    → JSX→JS (oxc) → execute in Deno Core → html | javascript | schema | json
```

Non-obvious constraints to respect:

- **Renderer pool is thread-local** (`core/src/renderer/pool.rs`) because V8 isolates
  are not Send/Sync. Don't try to share runtimes across threads.
- **JS engine assets are embedded** via `include_str!` (`core/static/`) and extracted
  to a temp dir on first use.
- **Resource limits are load-bearing** (batch size, content/component size, cached
  renderers) — they exist to prevent memory exhaustion; don't lift them casually.
- Version lives in `VERSION` and is synced everywhere by `release.py`; never bump
  version fields by hand.
- Install pre-commit hooks with `./hooks/install.sh` (fmt, clippy, core tests).

## The workflow

This project drives all work through **OpenSpec**, augmented with an abstraction-layer
discipline and a build-vs-adopt gate. Follow the pipeline — the rules apply themselves.

```
/opsx:explore   Think it through. Decompose (invariants, boundaries, ≥3 strategies). No code.
      ↓
/opsx:propose   Generate proposal.md (WHY/scope) + specs (abstract WHAT) + design.md (HOW).
      ↓
/opsx:decide    Build-vs-adopt gate: per critical concern, Rent>Adopt>Extend>Fork>Build.
      ↓             Records each decision into design.md. Run before implementing.
/opsx:apply     Implement the tasks. Thin entry points; adapters isolate every dependency.
      ↓
/opsx:sync      Fold the change's delta specs into the main specs.
      ↓
/opsx:archive   Close out the completed change.
```

## The two ideas that make this work

1. **Abstraction layers stay separate.** WHAT (`specs/`) is language-agnostic behavior.
   HOW (`design.md`) is structure + tool choices. DO (`tasks.md` + code) is the implementation.
   No layer leaks into another. Core holds behavior; every surface (CLI/GUI/API) is a thin adapter.

2. **Adopt before you build.** For anything correctness-, security-, or reliability-critical,
   prefer a mature tool over hand-writing it. `/opsx:decide` makes that call explicit and records it.

## Where the rules live (don't restate them)

- **`openspec/config.yaml`** — the philosophy, injected once into every artifact by the CLI.
- **`openspec/guidelines.md`** — the full reference: build-vs-adopt hierarchy, maturity rubric, doc taxonomy.

Commands stay thin and point at these, so a change costs few tokens to plan.

## Keep the main thread cheap

For broad or exploratory searches — locating code across many files, surveying naming
conventions, answering a question that spans several files — delegate to the `Explore` or
`general-purpose` subagent and keep only its conclusion. Don't read the file dumps into the
main thread. For a single known file or symbol, just search directly (a subagent would cost
more than it saves).

## Where findings go (promote or discard — never accumulate)

There is **no `/research` folder**. A global, ever-growing notes dump rots and starts
misleading. Every durable finding gets exactly one canonical home; everything else dies in
the scratchpad. When a subagent surfaces something worth keeping, route it:

- **Throwaway exploration** → the session scratchpad. Auto-discarded; never committed.
- **Derivable from code** → leave it in the code. Don't snapshot it.
- **A decision + its why** (build-vs-adopt, tradeoffs) → `design.md` ADR block.
- **Durable behavior contract** → `specs/` (synced to main specs on `/opsx:sync`).
- **In-flight notes for a specific change** → `openspec/changes/<change>/`, archived on `/opsx:archive`.
- **Non-derivable fact about the user/project** → the memory dir; update or delete when wrong.

The discipline is promote-or-discard, not save-more: one home, one lifecycle, one owner.
