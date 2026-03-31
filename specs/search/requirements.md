---
spec: search.spec.md
---

## User Stories

- As a user, I want to search for ASCII strings in a binary file so that I can find text content
- As a user, I want to search for hex byte patterns so that I can locate specific byte sequences
- As a user, I want to navigate between search results so that I can examine each match

## Acceptance Criteria

- Search supports ASCII string queries and hex byte patterns (e.g., `FF D8 FF`)
- Case-insensitive search is available for ASCII queries
- All matches in the buffer are found and navigable with next/previous
- Search results update the cursor position and scroll the view to the match
- Invalid hex patterns produce a clear error message

## Constraints

- Full-buffer search must complete fast enough to feel interactive on typical files

## Out of Scope

- Regex search
- Search and replace (edit mode handles byte modification)
