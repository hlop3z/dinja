const engine = {
  Fragment: Symbol("Fragment"),

  render(component, context = {}) {
    return JSON.stringify(component(context));
  },

  h(type, props, ...children) {
    return {
      type: type === this.Fragment ? "Fragment" : type,
      props: props || null,
      children: flatten(children).map(normalizeChild),
    };
  },
};

// Ensures children don't nest arrays
function flatten(arr) {
  return arr.flat(Infinity);
}

// Wraps text nodes in a standard structure
function normalizeChild(child) {
  if (child == null || child === false) return null;

  if (typeof child === "string" || typeof child === "number") {
    return { type: "text", value: child };
  }

  return child;
}

export default engine;
