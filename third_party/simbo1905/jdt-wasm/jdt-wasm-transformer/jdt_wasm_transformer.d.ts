/**
 * JDT WASM Transformer
 * 
 * WebAssembly bindings for JSON Document Transforms (JDT)
 */

/**
 * Get the version of the JDT WASM transformer
 */
export function version(): string;

/**
 * Apply a JDT transform to a source JSON document.
 * 
 * @param source_json - The source JSON document as a string
 * @param transform_json - The JDT transform specification as a string
 * @returns The transformed JSON as a string
 * @throws Error if transformation fails
 * 
 * @example
 * ```typescript
 * const source = JSON.stringify({ name: "example", version: "1.0.0" });
 * const transform = JSON.stringify({ version: "2.0.0" });
 * const result = transform(source, transform);
 * console.log(JSON.parse(result));
 * // { name: "example", version: "2.0.0" }
 * ```
 */
export function transform(source_json: string, transform_json: string): string;

/**
 * Apply a JDT transform with pretty-printed output.
 * 
 * Same as `transform()` but returns formatted JSON with indentation.
 * 
 * @param source_json - The source JSON document as a string
 * @param transform_json - The JDT transform specification as a string
 * @returns The transformed JSON as a pretty-printed string
 * @throws Error if transformation fails
 */
export function transform_pretty(source_json: string, transform_json: string): string;

/**
 * Validate a JDT transform specification without applying it.
 * 
 * @param transform_json - The JDT transform specification as a string
 * @throws Error if validation fails
 * 
 * @example
 * ```typescript
 * try {
 *   validate_transform('{"@jdt.remove": ["password"]}');
 *   console.log("Transform is valid");
 * } catch (e) {
 *   console.error("Invalid transform:", e);
 * }
 * ```
 */
export function validate_transform(transform_json: string): void;

/**
 * Check if a string is valid JSON
 * 
 * @param json_str - String to validate
 * @returns true if valid JSON, false otherwise
 */
export function is_valid_json(json_str: string): boolean;
