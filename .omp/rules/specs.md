---
description: Maintain and synchronize product specs and tech specs with code changes. Never allow specifications and implementation to drift.
alwaysApply: true
---

# Specifications Synchronization (Specs)

Maintain and continuously synchronize product specifications and technical specifications with code changes. Code and specs must remain in lockstep.

## 1. Specification Types

- **Product Specifications**: Describe user-facing behavior, capabilities, workflows, and functional requirements.
- **Technical Specifications**: Describe architectural decisions, module contracts, interfaces, data layouts, and system invariants.
  - Architectural decisions follow MADR (Markdown Architectural Decision Records):
    Reference: [.omp/rules/references/madr](references/madr)

## 2. Synchronization Invariants

- **Zero Drift**: Any commit modifying behavior, APIs, or contracts must update the corresponding product and technical specifications in the same commit.
- **Spec First or Lockstep**: Write or update specifications before or alongside implementation. Never postpone specification updates to follow-up work.
- **Truth Discipline**: Code that contradicts specification or specification that contradicts code is a defect.
- **Architectural Changes**: When an architectural choice changes, record or supersede the decision record according to the MADR template (`references/madr/template/adr-template.md`).
