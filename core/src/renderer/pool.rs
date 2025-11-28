//! Thread-local renderer pooling to avoid reloading static JS libraries on every request.
//!
//! ## Architecture
//!
//! This module implements a thread-local cache of JavaScript renderers using LRU (Least Recently Used)
//! eviction. Each thread maintains its own cache, preventing the need to reload engine and other
//! static libraries on every request.
//!
//! ## Thread Safety
//!
//! **Critical**: This module uses `thread_local!` because `JsRuntime` is not `Send` or `Sync`.
//! This means:
//!
//! - Each thread has its own independent cache
//! - Renderers cannot be shared across threads
//! - The pool automatically manages renderer lifecycle per thread
//!
//! ## Why Thread-Local Storage?
//!
//! Deno Core's `JsRuntime` is not thread-safe and cannot be shared across threads. Using
//! thread-local storage allows each thread to maintain its own cache of renderers without
//! requiring synchronization primitives like `Mutex`, which would serialize access and hurt
//! performance in a multi-threaded web server.
//!
//! ## Performance Considerations
//!
//! - Renderers are cached per profile (Engine)
//! - LRU eviction prevents unbounded memory growth
//! - Pool warming reduces first-request latency
//! - Maximum cache size per profile prevents excessive memory usage
//!
//! ## Example
//!
//! ```no_run
//! use dinja_core::renderer::pool::{RendererPool, RendererProfile};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let pool = RendererPool::new("static", 4);
//! pool.warm(1); // Pre-create renderers for common profiles
//!
//! let lease = pool.checkout(RendererProfile::Engine)?;
//! let renderer = lease.renderer()?;
//! // Use renderer...
//! // Renderer is automatically returned to pool when lease is dropped
//! # Ok(())
//! # }
//! ```
use super::JsRenderer;
use crate::error::MdxError;
use anyhow::Result as AnyhowResult;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::path::PathBuf;
use std::thread_local;

/// Cache entry tracking renderers and their access order for LRU eviction
struct CacheEntry {
    /// Stack of available renderers (most recently used at the end)
    renderers: VecDeque<JsRenderer>,
}

impl CacheEntry {
    fn new() -> Self {
        Self {
            renderers: VecDeque::new(),
        }
    }

    /// Pops the most recently used renderer (LRU: remove from front)
    fn pop(&mut self) -> Option<JsRenderer> {
        self.renderers.pop_back()
    }

    /// Pushes a renderer, evicting the least recently used if at capacity
    fn push_with_limit(&mut self, renderer: JsRenderer, max_size: usize) {
        // If at capacity, remove least recently used (front of deque)
        if self.renderers.len() >= max_size {
            let _ = self.renderers.pop_front();
        }
        // Add most recently used to the back
        self.renderers.push_back(renderer);
    }

    fn len(&self) -> usize {
        self.renderers.len()
    }
}

impl Drop for CacheEntry {
    fn drop(&mut self) {
        // V8 isolates must be dropped LIFO (reverse creation order) or the runtime panics.
        while self.renderers.pop_back().is_some() {}
    }
}

thread_local! {
    static RENDERER_CACHE: RefCell<HashMap<RendererKey, CacheEntry>> =
        RefCell::new(HashMap::new());
}

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
enum RendererKey {
    Engine,
}

/// Profiles describe the runtime flavor required for a given render request.
#[derive(Clone, Copy)]
pub enum RendererProfile {
    /// Standard engine renderer used for HTML and JavaScript outputs.
    Engine,
}

impl RendererProfile {
    fn key(self) -> RendererKey {
        match self {
            RendererProfile::Engine => RendererKey::Engine,
        }
    }
}

/// Lease that returns the renderer to the cache when dropped.
pub struct RendererLease<'pool> {
    renderer: Option<JsRenderer>,
    key: RendererKey,
    pool: &'pool RendererPool,
}

impl<'pool> RendererLease<'pool> {
    /// Returns a reference to the leased renderer.
    ///
    /// # Errors
    /// Returns an error if the renderer has already been returned to the pool.
    pub fn renderer(&self) -> Result<&JsRenderer, MdxError> {
        self.renderer
            .as_ref()
            .ok_or_else(|| MdxError::tsx_transform("Renderer already returned to pool"))
    }
}

impl<'pool> Drop for RendererLease<'pool> {
    fn drop(&mut self) {
        if let Some(renderer) = self.renderer.take() {
            self.pool.return_renderer(self.key, renderer);
        }
    }
}

/// Thread-local cache of initialized JavaScript runtimes.
///
/// This pool uses LRU (Least Recently Used) eviction to manage cached renderers.
/// Each thread maintains its own cache, so renderers are not shared across threads.
/// This is necessary because `JsRuntime` is not `Send` or `Sync`.
///
/// The cache automatically evicts the least recently used renderer when the
/// maximum cache size is reached for a given profile.
#[derive(Clone)]
pub struct RendererPool {
    static_dir: PathBuf,
    max_cached_per_key: usize,
}

impl RendererPool {
    /// Creates a new renderer pool.
    ///
    /// # Arguments
    /// * `static_dir` - Directory containing static JavaScript files
    /// * `max_cached_per_key` - Maximum number of cached renderers per profile
    ///
    /// # Returns
    /// A new `RendererPool` instance
    pub fn new(static_dir: impl Into<PathBuf>, max_cached_per_key: usize) -> Self {
        Self {
            static_dir: static_dir.into(),
            max_cached_per_key,
        }
    }

    /// Warms up the pool by pre-creating renderers for common profiles.
    ///
    /// This reduces first-request latency by initializing renderers ahead of time.
    /// Errors during warming are logged but don't prevent pool creation.
    ///
    /// # Arguments
    /// * `warm_count` - Number of renderers to pre-create per profile (defaults to 1)
    pub fn warm(&self, warm_count: usize) {
        if warm_count == 0 {
            return;
        }

        // Warm up common profiles
        let profiles = [RendererProfile::Engine];

        for profile in profiles.iter() {
            for _ in 0..warm_count.min(self.max_cached_per_key) {
                if let Ok(renderer) = self.create_renderer(*profile) {
                    let key = profile.key();
                    self.return_renderer(key, renderer);
                } else {
                    // Log but continue - warming is best-effort
                    eprintln!("Warning: Failed to warm renderer for profile Engine");
                }
            }
        }
    }

    /// Checks out a renderer from the pool for the given profile.
    ///
    /// The renderer is returned to the pool when the `RendererLease` is dropped.
    ///
    /// # Arguments
    /// * `profile` - The renderer profile (Engine)
    ///
    /// # Returns
    /// A `RendererLease` containing the renderer, or an error if creation fails
    pub fn checkout<'pool>(
        &'pool self,
        profile: RendererProfile,
    ) -> AnyhowResult<RendererLease<'pool>> {
        let key = profile.key();
        let renderer =
            Self::take_cached_renderer(key).map_or_else(|| self.create_renderer(profile), Ok)?;

        Ok(RendererLease {
            renderer: Some(renderer),
            key,
            pool: self,
        })
    }

    fn create_renderer(&self, profile: RendererProfile) -> AnyhowResult<JsRenderer> {
        match profile {
            RendererProfile::Engine => JsRenderer::new(&self.static_dir),
        }
    }

    fn take_cached_renderer(key: RendererKey) -> Option<JsRenderer> {
        RENDERER_CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            let entry = cache.get_mut(&key)?;
            let renderer = entry.pop()?;
            // Remove entry if empty to prevent cache bloat
            if entry.len() == 0 {
                cache.remove(&key);
            }
            Some(renderer)
        })
    }

    fn return_renderer(&self, key: RendererKey, renderer: JsRenderer) {
        RENDERER_CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            let entry = cache.entry(key).or_insert_with(CacheEntry::new);
            // Use LRU eviction: remove oldest if at capacity
            entry.push_with_limit(renderer, self.max_cached_per_key);
        });
    }
}
