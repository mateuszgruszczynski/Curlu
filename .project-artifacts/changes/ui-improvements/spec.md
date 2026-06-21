# Spec: UI Improvements

## Context
Rust egui desktop HTTP client (curlu). Single-file UI in `src/app.rs`, HTTP logic in `src/http.rs`. No tests exist. The app stores requests as `.curl` files and already has `SavedRequest::from_curl` parser and `to_curl` exporter.

---

## Change 1 — File Browser button moved to left of menu bar

**Before:** Menu order: New Request | Open | Save | Save As | --- | [File Browser checkbox] | --- | Show as curl

**After:** Menu order: [File Browser checkbox] | --- | New Request | Open | Save | Save As | --- | Show as curl | Import curl

The checkbox toggles the left side panel. Moving it to the far left makes it a panel toggle like a typical IDE sidebar button.

**AC-1:** The "File Browser" checkbox/toggle is the leftmost element in the top menu bar.

---

## Change 2 — Import request from curl string

A new "Import curl…" button in the menu bar (right side, next to "Show as curl"). Clicking it opens a modal window where the user can paste a raw curl command string. Clicking "Import" parses the string via the existing `SavedRequest::from_curl` and populates the current request fields (method, URL, headers, body). Clicking "Cancel" or the window close button dismisses without change.

**AC-2a:** "Import curl…" button exists in the menu bar.
**AC-2b:** Clicking it opens a window with a multi-line editable text area (hint: "Paste curl command…").
**AC-2c:** Clicking "Import" parses the curl string and populates method/URL/headers/body. If parsing fails (text is not a valid curl command), the window stays open with an error message.
**AC-2d:** The import text area is cleared when the window is opened fresh.

---

## Change 3 — File browser filter with debounce

A text input appears inside the side panel, above the directory tree, as a filter box. As the user types, the tree is filtered to show only files (`.curl`) whose name contains the filter string (case-insensitive). Directories are shown only if they contain at least one matching descendant; matching directories auto-expand. Filter is debounced: the tree updates 300 ms after the last keystroke.

**AC-3a:** A single-line filter text input is rendered at the top of the file browser side panel.
**AC-3b:** The file tree updates 300 ms after the last keystroke (no update on every character).
**AC-3c:** Files whose name does not contain the filter string (case-insensitive) are hidden.
**AC-3d:** Directories with no matching descendants are hidden. Directories with matching descendants are auto-expanded.
**AC-3e:** When the filter is empty the tree displays normally (no filtering).

---

## Change 4 — JSON path filter on response body

A text input appears in the response column, between the "Response Body" label and the body text area. When non-empty it treats the response body as JSON, evaluates a dot-path expression (e.g. `data.items`, `data.items[0]`, `$.results`), and displays only the matched sub-value (pretty-printed). When the path matches nothing or the body is not valid JSON, displays `<no match>`. The filter persists across request resends — a new response is automatically filtered by the currently entered path.

Path syntax supported: optional leading `$`, dot-separated keys, bracket array indices. Examples:
- `data` → `/data`
- `data.items` → `/data/items`
- `data.items[0]` → `/data/items/0`
- `$.results[2].name` → `/results/2/name`

Implemented using `serde_json::Value::pointer()` (JSON Pointer, RFC 6901) with a lightweight converter for dot-notation input. No new crate dependency needed.

**AC-4a:** A text input labeled "Filter (JSON path):" appears between "Response Body" label and the body area.
**AC-4b:** When the filter is empty, the full response body is shown (unchanged behavior).
**AC-4c:** When non-empty and body is valid JSON and path matches, the matched sub-value is shown pretty-printed.
**AC-4d:** When path matches nothing or body is not JSON, the body area shows `<no match>`.
**AC-4e:** The filter is preserved after re-sending the request (new response is re-filtered automatically).
**AC-4f:** The filter input is visible only in the response column (not the request column).

---

## Out of scope
- Persistent storage of the JSON path filter across app restarts
- JSONPath wildcard/recursive descent operators (`..`, `*`, `?()`)
- Regex filtering in file browser
- Persistent file browser filter across sessions
