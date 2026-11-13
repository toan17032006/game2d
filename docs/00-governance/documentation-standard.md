# Documentation Standard

- Owner: Engineering Enablement
- Status: `Approved`
- Review: `2027-05-13`

## Rules

- MUST use UTF-8.
- MUST use `kebab-case.md` for file names.
- MUST use numbered folders for lifecycle order.
- MUST start each document with `Owner`, `Status`, `Last updated` and `Review date`.
- MUST use `Draft`, `In review`, `Approved` or `Deprecated`.
- MUST include `## References`.
- MUST use repository-root paths for internal references.
- MUST include name, URL, version and access date for external references.
- MUST move obsolete documents to `/docs/99-archive/`.
- MUST NOT store secrets, credentials, private keys or production data.
- Agent MUST treat project documentation Markdown as read-only.
- Agent MAY create and update Markdown only inside `/.agent/tasks/<TASK-ID>/` for the current task.
- Agent MUST NOT modify project documentation from a task workflow unless the User directly requests it.

## Required order

1. Title
2. Metadata
3. `## Purpose` or `## Rules`
4. Content
5. `## References`

## References

- `/AGENTS.md`
- `/docs/README.md`
- `/docs/09-reference/README.md`
