const { Renderer } = require('./wrapper.js');

console.log('Creating renderer...');
const renderer = new Renderer();

console.log('\nTest 1: Simple MDX to HTML');
const result1 = renderer.render({
  settings: { output: 'html', minify: false },
  mdx: { 'test.mdx': '# Hello **World**' }
});

console.log('Total:', result1.total);
console.log('Succeeded:', result1.succeeded);
console.log('Status:', result1.files['test.mdx'].status);
console.log('Output:', result1.files['test.mdx'].result.output);

console.log('\nTest 2: Multiple files');
const result2 = renderer.render({
  settings: { output: 'html', minify: false },
  mdx: {
    'file1.mdx': '# File 1',
    'file2.mdx': '# File 2'
  }
});

console.log('Total:', result2.total);
console.log('Succeeded:', result2.succeeded);
console.log('File 1 status:', result2.files['file1.mdx'].status);
console.log('File 2 status:', result2.files['file2.mdx'].status);

console.log('\nTest 3: Different output formats');
const result3schema = renderer.render({
  settings: { output: 'schema', minify: false },
  mdx: { 'test.mdx': '# Hello' }
});

console.log('Schema status:', result3schema.files['test.mdx'].status);
console.log('Schema output length:', result3schema.files['test.mdx'].result.output.length);

const result3json = renderer.render({
  settings: { output: 'json', minify: false },
  mdx: { 'test.mdx': '# Hello' }
});

console.log('JSON status:', result3json.files['test.mdx'].status);
console.log('JSON output preview:', result3json.files['test.mdx'].result.output.substring(0, 100) + '...');

console.log('\nAll tests completed successfully!');
