# Contributing to Iterum

This small document contains the guidelines for contributing to Iterum,
especially through bug reports, feature proposals, and pull requests.

Iterum is an open-source game engine focused on large open worlds.
The project is still in early active development,
so its architecture and APIs may change significantly.

## Table of contents

- [Reporting Bugs](#reporting-bugs)
- [Proposing Features and Improvements](#proposing-features-and-improvements)
- [Contributing Pull Requests](#contributing-pull-requests)
- [Style](#style)
- [Communication](#communication)

## Reporting Bugs

Before reporting a bug, check whether it has already been reported.

A bug report should include a clear description of the issue,
what you expected to happen, and how to reproduce it.

Include relevant logs, error messages,
or other information that may help reproduce the issue.

For issues that may be system-dependent, include relevant system information
such as your operating system, GPU, and Iterum version or commit.

## Proposing Features and Improvements

Feature proposals and improvements are welcome.

Before proposing something new,
check existing issues and discussions to see whether it has already been suggested.

Explain what you want to change,
why it would be useful, and how you expect it to work when possible.

Larger changes to Iterum's architecture or core systems
should generally be discussed before implementation.

## Contributing Pull Requests

Pull requests should focus on a specific change and avoid unrelated modifications.

Before opening a pull request,
make sure your changes build successfully and have been tested where applicable.

Keep pull requests reasonably small and focused so they can be reviewed more easily.

The pull request description should explain what was changed and why.

For larger changes, discussing the approach before opening the pull request is recommended.

## Style

Keep commit messages and pull request titles simple and descriptive.

For example:

```txt
Add: Screen Space Reflections
Change: Shadow Culling
Fix: Camera Jitter
Remove: Legacy Renderer
```

Avoid vague or unnecessarily long descriptions.

Code should generally follow the existing style of the project.
Avoid unnecessary abstractions or dependencies when they are not needed.

## Communication

Use GitHub Issues for bugs and specific proposed changes.

Use GitHub Discussions for questions, ideas, and general discussion.

Please keep discussions constructive and relevant to Iterum.
