---
description: Document code intent clearly when code alone does not explain it, linking comments to relevant specifications.
alwaysApply: true
---

# Code Intent and Documentation

Ensure code explicitly conveys why it exists and why specific choices, invariants, or boundaries were selected.

Reference: [Specs Skill](../../.omp/skills/specs/SKILL.md)

## 1. Express Intent First in Code

- **Self-Explaining Design**: Use precise domain naming, cohesive types, and clear function signatures so that the code self-evidently conveys *what* it does.
- **Do Not Narrate Mechanics**: Avoid redundant comments that merely paraphrase what the code already shows.

## 2. Commenting Intent

- **Explain the "Why"**: When code alone cannot explain the rationale behind a decision, constraint, edge-case mitigation, performance trade-off, or non-obvious invariant, add concise inline or doc comments explaining the intent.
- **Link to Specifications**:
  - Where a decision, business rule, or architectural constraint originates from formal product or technical specifications, link the comment directly to the specification or MADR decision record.
  - Keep comments synchronized with specifications using the `specs` skill ([.omp/skills/specs/SKILL.md](../../.omp/skills/specs/SKILL.md)). If specifications or architectural decisions change, update corresponding intent comments in the same commit.
