const { Renderer: NativeRenderer } = require('./index.js');

// Wrapper class that handles JSON serialization
class Renderer {
  constructor() {
    this._native = new NativeRenderer();
  }

  render(input) {
    const inputJson = JSON.stringify(input);
    const outputJson = this._native.render(inputJson);
    return JSON.parse(outputJson);
  }
}

module.exports = {
  Renderer
};
