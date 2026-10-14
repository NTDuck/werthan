---
name: specs
description: Maintains and synchronizes product specs and technical specifications with code changes. Use whenever writing features, refactoring, or changing interfaces.
---

# Specs

Maintains and continuously synchronizes product specifications and technical specifications with code changes.

## Workflow

1. **Identify Spec Surface**: On any change, determine affected product behavior and technical contracts.
2. **Review Existing Specs & Decisions**: Check existing product requirements and MADR architectural decision records under `.omp/rules/references/madr/`.
3. **Keep In Lockstep**: Update product specs and technical specifications alongside code modifications in the same change set.
4. **Audit for Drift**: Verify that no unverified claims or stale specifications remain before yielding.
