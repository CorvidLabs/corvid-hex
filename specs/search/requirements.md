---
spec: search.spec.md
---

## User Stories

- As a user, I want to search for ASCII strings in a binary file so that I can find text content
- As a user, I want to search for hex byte patterns so that I can locate specific byte sequences
- As a user, I want to navigate between search results so that I can examine each match

## Acceptance Criteria

Stable, testable requirements are listed by ID after the scope sections.

## Constraints

- Full-buffer search must complete fast enough to feel interactive on typical files

## Out of Scope

- Regex search
- Search and replace (edit mode handles byte modification)

### REQ-search-001

Search SHALL support ASCII string queries and hexadecimal byte patterns such as `FF D8 FF`.

Acceptance Criteria
- Representative ASCII and hexadecimal queries return their matching offsets.

### REQ-search-002

Search SHALL provide case-insensitive matching for ASCII queries.

Acceptance Criteria
- A case-insensitive query matches equivalent ASCII text with different letter case.

### REQ-search-003

Search SHALL find all matches in the buffer and allow next and previous navigation among them.

Acceptance Criteria
- The result list includes every occurrence and navigation cycles through the list.

### REQ-search-004

Selecting a search result SHALL move the cursor to the match and scroll it into view.

Acceptance Criteria
- Activating a result updates cursor and viewport to its starting offset.

### REQ-search-005

Search SHALL report a clear error for invalid hexadecimal patterns.

Acceptance Criteria
- A malformed hex query returns an explanatory error without starting a search.
