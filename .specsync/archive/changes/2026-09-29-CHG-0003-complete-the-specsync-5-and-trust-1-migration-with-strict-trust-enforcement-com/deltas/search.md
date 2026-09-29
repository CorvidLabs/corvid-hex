## ADDED
### REQUIREMENT REQ-search-001
Search SHALL support ASCII string queries and hexadecimal byte patterns such as `FF D8 FF`.

Acceptance Criteria
- Representative ASCII and hexadecimal queries return their matching offsets.

### REQUIREMENT REQ-search-002
Search SHALL provide case-insensitive matching for ASCII queries.

Acceptance Criteria
- A case-insensitive query matches equivalent ASCII text with different letter case.

### REQUIREMENT REQ-search-003
Search SHALL find all matches in the buffer and allow next and previous navigation among them.

Acceptance Criteria
- The result list includes every occurrence and navigation cycles through the list.

### REQUIREMENT REQ-search-004
Selecting a search result SHALL move the cursor to the match and scroll it into view.

Acceptance Criteria
- Activating a result updates cursor and viewport to its starting offset.

### REQUIREMENT REQ-search-005
Search SHALL report a clear error for invalid hexadecimal patterns.

Acceptance Criteria
- A malformed hex query returns an explanatory error without starting a search.

