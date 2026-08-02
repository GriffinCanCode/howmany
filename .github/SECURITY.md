# Security Policy

## Supported versions

| Version | Supported |
| ------- | --------- |
| 2.1.x   | Yes       |
| < 2.1   | No        |

## Reporting a vulnerability

Please **do not** open a public issue for a security problem.

Report privately through
[GitHub Security Advisories](https://github.com/GriffinCanCode/howmany/security/advisories/new),
or by email to griffin@griffin-code.com.

Include the version, your platform, reproduction steps, and the impact you
believe the issue has.

You can expect an acknowledgement within 72 hours and a status update within
seven days. If the report is confirmed, a fix will be released and you will be
credited in the advisory unless you ask otherwise.

## Scope

`howmany` reads source files from disk and writes reports. The areas most
relevant to security are path traversal while walking a directory tree, output
escaping in generated HTML reports, and resource exhaustion on hostile inputs
(deeply nested directories, symlink loops, very large files).
