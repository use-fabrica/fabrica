# Issue Tracker: Local Markdown

Issues and specs live under `.scratch/<feature-slug>/`.

- Specs: `spec.md`.
- Tickets: `issues/<NN>-<slug>.md`, one file per ticket, numbered from `01`.
- Triage: record the state in a `Status:` line near the top (see `triage-labels.md`).
- Conversation: append under `## Comments`.

To publish, create the file at its convention path. To fetch, read the referenced path or resolve the issue number within the named feature; ask if the number is ambiguous.

## Wayfinding

- Map: `.scratch/<effort>/map.md` (Notes, Decisions-so-far, Fog).
- Child tickets: `.scratch/<effort>/issues/<NN>-<slug>.md`, numbered from `01`, with the question in the body and a `Type:` of `research`, `prototype`, `grilling`, or `task`.
- Dependencies: `Blocked by: NN, NN`; a ticket is unblocked when every listed ticket is `resolved`.
- Pick the lowest-numbered open, unblocked, unclaimed ticket. Set `Status: claimed` before starting.
- To resolve, append `## Answer`, set `Status: resolved`, and add a summary and link under Decisions-so-far in the map.
