var engine = {
  Fragment: Symbol("Fragment"),

  render(component, context = {}) {
    const result = component(context);
    const cleaned = cleanInternalProps(result);
    return JSON.stringify(cleaned);
  },

  h(type, props, ...children) {
    return {
      type: type === this.Fragment ? "Fragment" : type,
      props: props || null,
      children: flatten(children).map(normalizeChild),
    };
  },
};

// Recursively removes all properties starting with "__" from objects
function cleanInternalProps(obj) {
  if (obj == null || typeof obj !== "object") {
    return obj;
  }

  if (Array.isArray(obj)) {
    return obj.map(cleanInternalProps);
  }

  const cleaned = {};
  for (const key in obj) {
    // Skip properties that start with "__"
    if (key.startsWith("__")) {
      continue;
    }
    // Recursively clean nested objects and arrays
    cleaned[key] = cleanInternalProps(obj[key]);
  }
  return cleaned;
}

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
