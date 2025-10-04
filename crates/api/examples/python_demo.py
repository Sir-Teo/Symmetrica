#!/usr/bin/env python3
"""
Demo of Symmetrica Python bindings

To run:
1. Build with: maturin develop
2. Run: python examples/python_demo.py
"""

import symmetrica as sym

def demo_basic():
    print("=== Basic Expressions ===")
    x = sym.Expr.sym("x")
    y = sym.Expr.sym("y")
    
    expr = x**2 + 3*x + 2
    print(f"Expression: {expr}")
    print(f"Simplified: {expr.simplify()}")
    print()

def demo_calculus():
    print("=== Calculus ===")
    x = sym.Expr.sym("x")
    
    # Differentiation
    expr = x**3 + 2*x**2 + x
    deriv = expr.diff("x")
    print(f"f(x) = {expr}")
    print(f"f'(x) = {deriv}")
    
    # Integration
    expr2 = x**2
    integral = expr2.integrate("x")
    print(f"∫{expr2} dx = {integral}")
    print()

def demo_solve():
    print("=== Equation Solving ===")
    x = sym.Expr.sym("x")
    
    # Solve x^2 - 5x + 6 = 0 (roots: x=2, x=3)
    expr = x**2 + sym.Expr.int(-5)*x + sym.Expr.int(6)
    print(f"Solving: {expr} = 0")
    roots = expr.solve("x")
    for i, root in enumerate(roots):
        print(f"  x_{i+1} = {root}")
    print()

def demo_functions():
    print("=== Mathematical Functions ===")
    x = sym.Expr.sym("x")
    
    # Trigonometric
    sin_x = sym.sin(x)
    print(f"sin(x) = {sin_x}")
    print(f"d/dx[sin(x)] = {sin_x.diff('x')}")
    
    # Exponential
    exp_x = sym.exp(x)
    print(f"exp(x) = {exp_x}")
    print(f"d/dx[exp(x)] = {exp_x.diff('x')}")
    
    # Logarithm
    ln_x = sym.ln(x)
    print(f"ln(x) = {ln_x}")
    print(f"d/dx[ln(x)] = {ln_x.diff('x')}")
    print()

def demo_substitution():
    print("=== Substitution ===")
    x = sym.Expr.sym("x")
    y = sym.Expr.sym("y")
    
    expr = x**2 + 2*x + 1
    print(f"Original: {expr}")
    
    # Substitute x with y+1
    result = expr.subs("x", y + sym.Expr.int(1))
    print(f"After x -> y+1: {result}")
    print(f"Simplified: {result.simplify()}")
    
    # Substitute with number
    result2 = expr.subs("x", sym.Expr.int(5))
    print(f"After x -> 5: {result2.simplify()}")
    print()

def demo_evaluation():
    print("=== Numerical Evaluation ===")
    x = sym.Expr.sym("x")
    
    expr = x**2 + sym.Expr.int(3)*x + sym.Expr.int(2)
    at_5 = expr.subs("x", sym.Expr.int(5))
    value = at_5.evalf()
    print(f"f(x) = {expr}")
    print(f"f(5) = {value}")
    print()

def demo_latex():
    print("=== LaTeX Export ===")
    x = sym.Expr.sym("x")
    expr = (x + sym.Expr.int(1))**2
    latex = expr.to_latex()
    print(f"Expression: {expr}")
    print(f"LaTeX: {latex}")
    print()

def demo_rational():
    print("=== Rational Numbers ===")
    half = sym.Expr.rat(1, 2)
    third = sym.Expr.rat(1, 3)
    
    sum_expr = half + third
    print(f"1/2 + 1/3 = {sum_expr.simplify()}")
    
    prod = half * third
    print(f"1/2 * 1/3 = {prod.simplify()}")
    print()

def main():
    print("Symmetrica Python Bindings Demo\n")
    
    demo_basic()
    demo_calculus()
    demo_solve()
    demo_functions()
    demo_substitution()
    demo_evaluation()
    demo_latex()
    demo_rational()
    
    print("Demo complete!")

if __name__ == "__main__":
    main()
