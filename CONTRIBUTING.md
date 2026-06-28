# Contributing to RDCS

Thank you for your interest in contributing to **RDCS (Remote Desktop Control System)**! We welcome contributions from developers of all experience levels. This guide will help you get started.

## Code of Conduct

By participating in this project, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md). Please read it before contributing. We are committed to providing a welcoming and inclusive experience for everyone.

## How to Report Bugs

If you encounter a bug, please help us by [opening a bug report](https://github.com/your-org/rdcs/issues/new?template=bug_report.md).

Before submitting a report:

1. **Search existing issues** to avoid duplicates.
2. **Reproduce the bug** reliably and document the exact steps.
3. **Include environment details** (OS, RDCS version, browser if applicable).
4. **Attach logs or screenshots** if they help illustrate the problem.

The more detail you provide, the faster we can investigate and fix the issue.

## How to Suggest Features

Have an idea for a new feature or improvement? We would love to hear it!

1. Open a [Feature Request](https://github.com/your-org/rdcs/issues/new?template=feature_request.md) describing the problem and your proposed solution.
2. Check the [Roadmap](docs/ROADMAP.md) to see if the feature is already planned.
3. Engage with the community in the issue thread to refine the idea before starting implementation.

## Development Setup

> **Note:** The RDCS tech stack is currently being designed. Detailed setup instructions will be added once the architecture is finalized. In the meantime, please refer to the [Roadmap](docs/ROADMAP.md) for the current project phase, or open a [Discussion](https://github.com/your-org/rdcs/discussions) if you would like to contribute to architecture decisions.

Once the stack is defined, this section will include:

- Prerequisites and system requirements
- Dependency installation steps
- Local development environment configuration
- Running the application and test suite

## Coding Standards

We use an `.editorconfig` file in the repository to enforce consistent code formatting across editors and IDEs. Please ensure your editor has an [EditorConfig plugin](https://editorconfig.org/) installed so that indentation, line endings, and other formatting rules are applied automatically.

General guidelines:

- Write clear, self-documenting code with meaningful variable and function names.
- Add comments only when the **why** is not obvious from the code itself.
- Keep functions and methods focused on a single responsibility.
- Avoid introducing new dependencies unless they solve a clearly defined problem.
- Run the linter and formatter before committing your changes.

## Commit Message Conventions

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification. Every commit message should be structured as follows:

```
<type>(<scope>): <short summary>

<optional body>

<optional footer>
```

### Types

| Type | Description |
| --- | --- |
| `feat` | A new feature |
| `fix` | A bug fix |
| `docs` | Documentation-only changes |
| `style` | Changes that do not affect the meaning of the code (white-space, formatting) |
| `refactor` | A code change that neither fixes a bug nor adds a feature |
| `perf` | A code change that improves performance |
| `test` | Adding or correcting tests |
| `chore` | Changes to the build process, CI, or auxiliary tools |
| `ci` | Changes to CI/CD configuration |
| `revert` | Reverting a previous commit |

### Examples

```
feat(relay): add WebSocket tunneling for remote sessions
fix(client): resolve connection timeout on slow networks
docs(readme): update installation instructions
chore(deps): bump protobuf version to 28.0
```

## Pull Request Process

1. **Fork the repository** and create a feature branch from `main`.
2. **Name your branch** descriptively, e.g., `feat/add-session-recording`.
3. **Make your changes**, committing with Conventional Commit messages.
4. **Write or update tests** for any new functionality or bug fixes.
5. **Update documentation** if your changes affect the public API or user-facing behavior.
6. **Run the full test suite** and ensure all checks pass.
7. **Submit your pull request** using the PR template. Fill in all required sections.
8. **Link related issues** in the PR description (e.g., `Fixes #42`).

Small, focused pull requests are reviewed faster than large ones. If your change is substantial, consider opening a discussion or issue first to get alignment on the approach.

## Review Process

- All pull requests require at least **one approval** from a maintainer before merging.
- Reviewers may request changes. Please respond to feedback promptly.
- Automated CI checks must pass before a PR can be merged.
- We aim to provide initial review feedback within **3 business days**.
- If your PR has not received attention after a week, feel free to ping a maintainer in the PR comments.

## Release Process

RDCS follows [Semantic Versioning (SemVer)](https://semver.org/):

- **MAJOR** version increments for incompatible API changes.
- **MINOR** version increments for backward-compatible new functionality.
- **PATCH** version increments for backward-compatible bug fixes.

Releases are cut from the `main` branch. The release process is managed by the core maintainers and involves tagging a version, generating changelogs from Conventional Commit history, and publishing artifacts.

## Questions?

If you have questions about contributing, feel free to:

- Open a [Discussion](https://github.com/your-org/rdcs/discussions) on GitHub.
- Reach out to a maintainer directly via an issue.

Thank you for helping make RDCS better!
