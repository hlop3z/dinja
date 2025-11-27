import engine from "./core";

export default function View(context = {}) {
  return engine.h(
    engine.Fragment,
    null,
    engine.h("h1", null, context.title),
    engine.h(
      "p",
      null,
      "By ",
      engine.h("strong", null, context.author),
      " on ",
      context.date
    ),
    engine.h(
      "p",
      null,
      "MDX is a powerful format that combines ",
      engine.h("strong", null, "Markdown"),
      " with ",
      engine.h("strong", null, "JSX"),
      "."
    ),
    engine.h("h2", null, "What is MDX?"),
    engine.h(
      "Slot",
      { if: "user.role === (`admin`)" },
      engine.h("div", null, " some conditional content ")
    ),
    engine.h(
      "Section",
      null,
      "MDX stands for Markdown + JSX. It's a format that lets you seamlessly write JSX in your Markdown documents.",
      engine.h(
        "Quote",
        null,
        "MDX gives you the best of both worlds: the simplicity of Markdown and the power of components."
      )
    ),
    engine.h("h2", null, "Benefits"),
    engine.h(
      "FeatureGrid",
      null,
      engine.h("Feature", {
        icon: "♻️",
        title: "Component Reusability",
        description: "Use the same components across multiple pages",
        click: "action.create.something",
      }),
      engine.h("Feature", {
        icon: "⚡",
        title: "Interactive Content",
        description: "Add interactivity to your documentation",
        click: "action.create.something",
      }),
      engine.h("Feature", {
        icon: "🔒",
        title: "Type Safety",
        description: "Get TypeScript support for your components",
        click: "action.create.something",
      })
    ),
    engine.h("h2", null, "Conclusion"),
    engine.h(
      "p",
      null,
      "MDX is perfect for documentation, blogs, and any content-heavy site where you need more than plain Markdown."
    ),
    engine.h(
      "p",
      null,
      engine.h("Button", { variant: "primary" }, "Learn More")
    )
  );
}
