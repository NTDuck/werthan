---
description: Conventional Commits 1.0.0 rules and format.
alwaysApply: true
---

# Conventional Commits

All commit messages must follow the [Conventional Commits 1.0.0](https://www.conventionalcommits.org/) specification.
Reference: [.omp/rules/references/conventionalcommits.org](references/conventionalcommits.org)

## Format

```text
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

## Types

- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation changes
- `style`: Changes that do not affect the meaning of the code
- `refactor`: Code changes that neither fix a bug nor add a feature
- `perf`: Code changes that improve performance
- `test`: Adding missing tests or correcting existing tests
- `build`: Changes that affect the build system or external dependencies
- `ci`: Changes to CI configuration files and scripts
- `chore`: Other changes that don't modify src or test files
- `revert`: Reverts a previous commit

## Rules

- Imperative, present tense subject ("add", not "added" or "adds").
- No trailing period in the subject line.
- Breaking changes indicated with `!` before `:` or via `BREAKING CHANGE:` footer.
