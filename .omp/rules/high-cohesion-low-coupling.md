---
description: Functions must be small, cohesive, maintain a single level of abstraction (Clean Code), and prefer functional programming paradigms.
alwaysApply: true
---

# High Cohesion, Loose Coupling

Design and construct software components, modules, and functions with high cohesion and loose coupling.

Reference: [High Cohesion, Loose Coupling](https://kevinsmith.io/high-cohesion-loose-coupling/) by Kevin Smith

## 1. Cohesion and Coupling Invariants

- **High Cohesion**: Elements within a module or function must work together to accomplish a single, well-bounded purpose. A cohesive function contains all the decision-making logic necessary to perform its job without leaking internal mechanics.
- **Loose Coupling**: Components must interact through clean, minimal interfaces so that changing one module or function does not cascade into required changes in other modules.

## 2. Function Design Principles

- **Small and Focused**: Functions should be small and do one thing well. If a function addresses multiple concerns, decompose it into smaller, cohesive helper functions.
- **Single Level of Abstraction (SLAP)**: Statements within a function body must operate at the same level of abstraction. High-level orchestrating functions coordinate flow, delegating detailed operations to lower-level functions.
- **Functional Programming Inside Functions**:
  - Prefer expression-oriented transformations and iterator pipelines (`map`, `filter`, `fold`, `find`, `collect`) over imperative loops and accumulator variables.
  - Prefer immutable values and pure transformations. Avoid unnecessary mutable state, reassignments, and hidden side effects within function bodies.
  - Emphasize explicit data flow, chaining transformations so inputs flow predictably into outputs.
