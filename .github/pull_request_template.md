## Scope

Describe the bounded implementation included in this Pull Request.

## Linked phase

- Execution pack / phase:
- Branch:

## Changed files

- Added:
- Modified:
- Deleted:

## Architecture decisions

Summarize decisions and link the relevant documents.

## Validation performed

```text
Paste exact commands and outcomes.
```

## Test output

```text
Paste concise automated test output.
```

## Screenshots

Not applicable for database-only phases, or attach relevant evidence.

## Security and data considerations

- [ ] No secrets, credentials, real `.env` files, production databases, or customer data added.
- [ ] No external service or telemetry introduced.
- [ ] Fixed-point and immutability rules remain intact.

## Known limitations

List deferred application-service invariants, CI limitations, and follow-up risks.

## Reviewer checklist

- [ ] Scope matches the active execution pack.
- [ ] Blueprint was preserved.
- [ ] Migrations build a fresh SQLite database.
- [ ] Foreign-key and no-`REAL` checks pass.
- [ ] Immutability and journal-balance tests pass.
- [ ] Partial document lineage is modeled.
- [ ] Documentation matches implementation.
- [ ] Pull Request remains Draft and unmerged.
