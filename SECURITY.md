# POSMAN security policy

## Supported release

Security fixes target the current POSMAN `1.x` stable release for Windows 10/11
64-bit. A supported installation remains offline by default and stores business
data locally under `%LOCALAPPDATA%\POSMAN`.

## Reporting a vulnerability

Do not publish credentials, customer data, production databases, recovery
codes, backup archives, or exploit details in a public issue. Use the
repository's private **Security → Report a vulnerability** channel when it is
available. If private reporting is unavailable, open a public issue containing
only a request for a private security contact and no sensitive detail.

Include the POSMAN version, Windows version, affected workflow, reproduction
conditions, and expected impact. Remove or replace all real business data.

## Security boundaries

- POSMAN has no cloud service, telemetry client, mandatory activation, or HTTP API.
- SQLite data, rendered documents, logs, exports, and backups stay beneath the
  local POSMAN data root unless the user explicitly exports a file.
- Passwords use Argon2id and recovery material is stored only as a one-time hash.
- Published and posted business records are corrected by reversal or compensating
  operations rather than destructive history edits.
- Release signing credentials are never committed to this repository.
