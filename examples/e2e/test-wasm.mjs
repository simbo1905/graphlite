/// Test the WASM validator outside a browser using bun/node.
/// Validates that the AOT-compiled WASM module produces correct errors.
import init, { validate } from '../../wasm-pkg/jtd_wasm_validator.js';

await init();

// Test instance with known errors (from the spec worked example)
const instance = JSON.stringify({
  name: "Alice",
  age: 300,
  tags: ["a", 42],
  extra: true
});

const errors = validate(instance);
console.log("Errors:", JSON.stringify(errors, null, 2));

// Verify expected errors
const expected = [
  { instancePath: "/age", schemaPath: "/properties/age/type" },
  { instancePath: "/tags/1", schemaPath: "/properties/tags/elements/type" },
  { instancePath: "/extra", schemaPath: "" },
];

let pass = true;
if (errors.length !== expected.length) {
  console.error(`Expected ${expected.length} errors, got ${errors.length}`);
  pass = false;
} else {
  for (let i = 0; i < expected.length; i++) {
    const a = errors.find(e => e.instancePath === expected[i].instancePath);
    if (!a) {
      console.error(`Missing error for instancePath: ${expected[i].instancePath}`);
      pass = false;
    } else if (a.schemaPath !== expected[i].schemaPath) {
      console.error(`Wrong schemaPath for ${expected[i].instancePath}: expected ${expected[i].schemaPath}, got ${a.schemaPath}`);
      pass = false;
    }
  }
}

// Test valid instance
const validErrors = validate(JSON.stringify({ name: "Bob", age: 25, tags: ["x"] }));
if (validErrors.length !== 0) {
  console.error(`Expected 0 errors for valid instance, got ${validErrors.length}`);
  pass = false;
}

if (pass) {
  console.log("WASM E2E TEST PASSED");
  process.exit(0);
} else {
  console.error("WASM E2E TEST FAILED");
  process.exit(1);
}
