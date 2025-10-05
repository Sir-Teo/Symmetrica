/* tslint:disable */
/* eslint-disable */
/**
 * Create common mathematical functions
 */
export function sin(x: Expr): Expr;
export function cos(x: Expr): Expr;
export function tan(x: Expr): Expr;
export function exp(x: Expr): Expr;
export function ln(x: Expr): Expr;
export function sqrt(x: Expr): Expr;
/**
 * A symbolic expression for WebAssembly
 */
export class Expr {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Create an integer expression
   */
  constructor(val: number);
  /**
   * Create a symbol expression
   */
  static symbol(name: string): Expr;
  /**
   * Create a rational expression
   */
  static rational(num: number, den: number): Expr;
  /**
   * Add two expressions
   */
  add(other: Expr): Expr;
  /**
   * Subtract two expressions
   */
  sub(other: Expr): Expr;
  /**
   * Multiply two expressions
   */
  mul(other: Expr): Expr;
  /**
   * Divide two expressions
   */
  div(other: Expr): Expr;
  /**
   * Raise expression to a power
   */
  pow(other: Expr): Expr;
  /**
   * Negate expression
   */
  neg(): Expr;
  /**
   * Simplify the expression
   */
  simplify(): Expr;
  /**
   * Differentiate with respect to a variable
   */
  diff(_var: string): Expr;
  /**
   * Integrate with respect to a variable
   */
  integrate(_var: string): Expr;
  /**
   * Substitute a symbol with another expression
   */
  subs(_var: string, val: Expr): Expr;
  /**
   * Solve equation for a variable (returns JSON array of solutions)
   */
  solve(_var: string): any;
  /**
   * Evaluate numerically (all symbols must be bound in the provided bindings JSON)
   */
  eval(bindings: any): number;
  /**
   * Convert to string representation
   */
  toString(): string;
  /**
   * Convert to LaTeX string
   */
  toLatex(): string;
  /**
   * Convert to S-expression string
   */
  toSExpr(): string;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_expr_free: (a: number, b: number) => void;
  readonly expr_new: (a: number) => number;
  readonly expr_symbol: (a: number, b: number) => number;
  readonly expr_rational: (a: number, b: number) => [number, number, number];
  readonly expr_add: (a: number, b: number) => [number, number, number];
  readonly expr_sub: (a: number, b: number) => [number, number, number];
  readonly expr_mul: (a: number, b: number) => [number, number, number];
  readonly expr_div: (a: number, b: number) => [number, number, number];
  readonly expr_pow: (a: number, b: number) => [number, number, number];
  readonly expr_neg: (a: number) => [number, number, number];
  readonly expr_simplify: (a: number) => [number, number, number];
  readonly expr_diff: (a: number, b: number, c: number) => [number, number, number];
  readonly expr_integrate: (a: number, b: number, c: number) => [number, number, number];
  readonly expr_subs: (a: number, b: number, c: number, d: number) => [number, number, number];
  readonly expr_solve: (a: number, b: number, c: number) => [number, number, number];
  readonly expr_eval: (a: number, b: any) => [number, number, number];
  readonly expr_toString: (a: number) => [number, number];
  readonly expr_toLatex: (a: number) => [number, number];
  readonly expr_toSExpr: (a: number) => [number, number];
  readonly sin: (a: number) => [number, number, number];
  readonly cos: (a: number) => [number, number, number];
  readonly tan: (a: number) => [number, number, number];
  readonly exp: (a: number) => [number, number, number];
  readonly ln: (a: number) => [number, number, number];
  readonly sqrt: (a: number) => [number, number, number];
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
