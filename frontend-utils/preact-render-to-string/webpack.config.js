const package = require("./package.json");

// webpack.config.js
const path = require("path");
const webpack = require("webpack"); // Required for BannerPlugin

// Define library name clearly
const LibraryName = package.name;
const FileName = `${package.name}.min.js`;

module.exports = {
  mode: "production", // Enables production optimizations (like minification)

  entry: "./src/index.js", // Your library's entry point

  output: {
    // Output file configuration
    filename: FileName, // Name of the output bundle
    path: path.resolve(__dirname, "dist"), // Output directory

    // Library configuration: How the library is exposed
    library: {
      name: LibraryName, // The name for the global variable (e.g., window.PREACT)
      type: "var", // Expose the library as a variable assignment (var PREACT = ...)
      export: "default", // Optional: Uncomment if your entry uses `export default`
      // and you want only the default export assigned to the global var.
    },

    // Ensure the bundle is wrapped in an IIFE.
    // This prevents internal variables from polluting the global scope
    // and ensures the library assignment code runs immediately.
    iife: true,

    // Clean the output directory before generating new files
    clean: true,
  },

  plugins: [
    // Use BannerPlugin ONLY for adding comments (like license or build info),
    // NOT for creating the IIFE structure itself.
    /*
    new webpack.BannerPlugin({
      // Example banner content - update with your actual info
      banner: `${LibraryName} Lite Bundle - Copyright ${new Date().getFullYear()} - MIT Licensed`,
      raw: false, // Set to false to let Webpack wrap the banner in `/* ...` comments, often preserved by minifiers.
      entryOnly: true, // Apply the banner only to the entry chunk file.
    }),
    */
  ],

  optimization: {
    // Minimization is enabled by default in production mode,
    // but explicitly setting it doesn't hurt.
    minimize: true,
  },

  resolve: {
    // Allows importing modules without specifying these extensions
    extensions: [".js", ".jsx", ".ts", ".tsx"], // Include extensions relevant to your project
  },

  // Optional: Add externals if your library depends on other global libraries
  // that should NOT be bundled (e.g., if PREACT relies on a global 'React').
  // externals: {
  //  'react': 'React', // Assumes 'React' is available globally
  // },

  // Optional: Configure source maps for debugging the minified code.
  // devtool: 'source-map', // Example: creates separate .map files
};
