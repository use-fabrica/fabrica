# Decisions

## 2026-04-04 Pre-Implementation
- Use custom `format_description!` for timestamp output (preserves `Z` suffix, not `+00:00`)
- Use `Rfc3339` for parsing only (accepts both `Z` and `+00:00`)
- Keep `last_opened` as `String` type (NOT `OffsetDateTime`)
- Keep `pub projects` field on `RecentProjects` struct (Option A: direct field, synced after mutation)
- `time` crate features: exactly `["parsing", "formatting", "macros"]`
- File organization: flat `.rs` files under `src/` (matches existing pattern)
- Handle future timestamps → `"just now"` via `elapsed.is_negative()`
