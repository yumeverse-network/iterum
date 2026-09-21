# Security Policy

## Supported Versions

Iterum is currently under active development.
Security fixes are applied to the current development version.

| Version        | Supported          |
| -------------- | ------------------ |
| `main`         | :white_check_mark: |
| Older versions | :x:                |

Once Iterum begins publishing stable releases,
this table will be updated to list the supported release versions.

## Reporting a Vulnerability

Please do not report security vulnerabilities through public GitHub issues,
discussions, pull requests, or other public channels.

Instead, use GitHub's private vulnerability reporting feature:

`Repository → Security and quality → Advisories → Report a vulnerability`

This allows the report to be submitted privately to the Iterum maintainers.

When reporting a vulnerability,
please include as much relevant information as possible, such as:

* A description of the vulnerability.
* The affected component or feature.
* Steps to reproduce the issue.
* The potential impact.
* A proof of concept, if available.
* The affected Iterum version or commit.

Reports will be reviewed and investigated privately. If the report is confirmed,
the maintainers will work on a fix and coordinate disclosure with the reporter
where appropriate.

Please do not publicly disclose the vulnerability until a fix
or other appropriate resolution is available.

## Scope

This policy covers security vulnerabilities in Iterum itself,
including the engine, editor, runtime, asset handling, networking,
and other code maintained in the Iterum repository.

Third-party dependencies should generally be reported to
their respective maintainers unless the issue is specifically caused
by how Iterum uses or integrates the dependency.

## Questions

For general bugs, crashes, feature requests, and other non-security issues,
please use the normal GitHub issue tracker.
