import express from "express";
import { createServer as createViteServer } from "vite";
import { Renderer } from "@dinja/core";

async function startServer() {
  const app = express();
  const renderer = new Renderer();

  // Create Vite server in middleware mode
  const vite = await createViteServer({
    server: { middlewareMode: true },
    appType: "spa",
  });

  // Parse JSON bodies
  app.use(express.json());

  // API endpoint for rendering MDX (server-side only)
  app.post("/api/render", (req, res) => {
    try {
      const { mdx, components, settings } = req.body;

      const result = renderer.render({
        settings: settings || { output: "html", minify: false },
        mdx: mdx || {},
        components: components || {},
      });

      res.json(result);
    } catch (error) {
      res.status(500).json({ error: error.message });
    }
  });

  // Use Vite's middleware for everything else
  app.use(vite.middlewares);

  app.listen(3000, () => {
    console.log("");
    console.log("  🦀 Dinja + Vite + Preact");
    console.log("");
    console.log("  → Local:   http://localhost:3000");
    console.log("  → API:     POST /api/render");
    console.log("");
  });
}

startServer();
