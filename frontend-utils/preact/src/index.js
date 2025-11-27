import { h, render } from "preact";

// Signals
import {
  signal,
  computed,
  effect,
  batch,
  useSignalEffect,
  useSignal,
  useComputed,
} from "@preact/signals";

// Hooks
import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "preact/hooks";

/**
 * useLifecycle - A reusable lifecycle hook for Preact
 *
 * @param {Object} hooks
 * @param {Function=} hooks.onCreated   - Called once when component is mounted
 * @param {Function=} hooks.onUpdated   - Called when dependencies change (or always if deps not specified)
 * @param {Function|AsyncFunction=} hooks.onRemoved   - Called on component unmount, supports async
 * @param {Array=} deps                - Dependency array to control updates
 */
function useLifecycle({ onCreated, onUpdated, onRemoved }, deps = undefined) {
  const isFirstRender = useRef(true);

  useEffect(() => {
    if (isFirstRender.current) {
      onCreated?.();
      isFirstRender.current = false;
    } else {
      onUpdated?.();
    }

    return () => {
      const maybePromise = onRemoved?.();
      if (maybePromise instanceof Promise) {
        maybePromise.catch(console.error); // Optional: handle unhandled promise
      }
    };
  }, deps);
}

function Fragment({ children }) {
  return children;
}

export default {
  // Custom
  useLifecycle,
  Fragment,

  // Preact Core
  h,
  render,

  // Hooks
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,

  // Signals
  signals: {
    signal,
    computed,
    effect,
    batch,
    useSignalEffect,
    useSignal,
    useComputed,
  },
};
