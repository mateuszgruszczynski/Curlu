# Curlu — Component Styling Guide (framework-agnostic / egui-oriented)

This guide describes **how each standard control should look and react**, in terms that map directly onto an immediate-mode toolkit like egui — not HTML/CSS. Instead of selectors and cascading rules, think in terms of **per-state visual sets**: every interactive widget is drawn from a small bundle of `{ fill, stroke, text-color, corner-radius }` chosen by its current interaction state.

---

## 1. The model: state-driven visual sets

Every interactive widget resolves to ONE visual set per frame, picked by interaction state:

| State | When | egui slot |
|---|---|---|
| **Resting** | idle, interactive | `widgets.inactive` |
| **Hovered** | pointer over it | `widgets.hovered` |
| **Active** | pressed / being dragged | `widgets.active` |
| **Selected / On** | toggle is on, row is selected | `selection` (bg_fill / stroke) |
| **Disabled / Static** | non-interactive text, separators | `widgets.noninteractive` |

A visual set holds: **background fill**, **stroke** (color + width), **text/foreground color**, **corner radius**, and **expansion** (egui can grow a widget a px on hover — keep this at 0 for this design; we want flat, stable edges).

Set these once globally (`ctx.set_visuals(...)`), then override per-widget only where this guide calls for it (e.g. the accent Send button, the focused field).

### Global baseline (applies to both themes; substitute the theme's token values)
- **Corner radius:** 4 px for controls, 5 px for content/scroll areas, 7 px for the window, 3 px for tiny chips/squares. Set a uniform 4 px on `widgets.*` and override the few exceptions.
- **Stroke width:** 1 px everywhere; **1.5 px** only for the focused input; **2 px** for accent ticks and the selected-row bar.
- **No shadows on panels/popups, no gradients, no hover-expansion.** egui defaults add a popup shadow — reduce it to near-zero for menus to stay consistent (a 1px stroke separates them instead).
- **Item spacing (density = compact):** ~6–8 px between controls in a row; row height for list items ~24–25 px; control height 26 (inputs) / 34–36 (request bar).
- **Text roles:** `text` = primary, `dim` = secondary/labels, `faint` = placeholders/captions. Apply per-widget via the foreground color of the visual set, or per-string where the widget paints its own label.

> Token values (light & dark, plus the 3 accents) are in `README.md → Design Tokens`. Below, names like `bg2`, `stroke`, `accent`, `dim` refer to those tokens.

---

## 2. Buttons

### 2a. Toolbar / text button (New Request, Open, Save, Show as curl…)
Borderless until hovered.
- Resting: fill = **none/transparent**, stroke = **none**, text = `text`, radius 4, inner padding ~4×9.
- Hovered: fill = `hover` (a low-alpha overlay: white@5% on dark, black@5% on light), stroke none, text `text`.
- Active (pressed): same fill, slightly stronger (or `stroke-soft` fill); no border.
- Implementation: a `Button` with `.frame(false)` normally, and a hovered weak background. In egui set `widgets.hovered.weak_bg_fill = hover`, `widgets.inactive.weak_bg_fill = transparent`, and zero bg_stroke on both.

### 2b. Primary / accent button (Send)
The only filled, colored control.
- Resting: fill = `accent` (base), stroke none, foreground = `on-accent text` (white for blue/green, near-black for amber), radius 4.
- Hovered: fill = `accent-hover`.
- Active: fill = `accent` (or 6% darker), no size change.
- Content: a play triangle (or label) painted in the on-accent color. Triangle is a filled shape — paint it directly; don't use an icon font.
- Implementation: build a `Button` and override its three `WidgetVisuals` fills locally (e.g. `ui.scope` + temporary visuals), since global widgets shouldn't all be accent-filled.

### 2c. Secondary / outline chip-button (the `{ }` prettify button)
- Resting: fill none, stroke = 1px `stroke`, text = `dim`, radius 4, padding 2×7.
- Hovered: stroke = `accent`, text = `accent`, fill still none.

### 2d. Window buttons (min / max / close)
- Square hit area 28×20, radius 4, glyph painted as a simple shape (bar / outlined square / ✕), color `dim`.
- Hovered: min/max → fill `hover`; **close → fill `#e5564a`, glyph white.**

---

## 3. Toggle controls

### 3a. Checkbox / on-toggle (File Browser toggle, "on" state)
- Box 13×13, radius 3. **On:** fill = `accent`, check mark in on-accent color. **Off:** fill = `bg2`, 1px `stroke`, empty.
- Label sits to the right in `text`.
- egui: `Checkbox`, with `selection.bg_fill = accent` and the check stroke in on-accent color; or paint a custom 13px box for exact sizing.

### 3b. Segmented / radio group (theme & accent pickers, if shown as a control)
- A row of equal segments, 1px `stroke` outline, radius 4, shared dividers.
- Selected segment: fill = `sel` (accent @ ~15% alpha), text = `accent`, weight up.
- Unselected: fill none, text = `dim`, hover fill `hover`.

---

## 4. Text inputs

### 4a. Standard field (Filter, JSON path, URL when unfocused)
- Fill = `bg2`, stroke = 1px `stroke`, radius 4, height 26 (compact 22 for inline ones).
- Text = `text`; **placeholder = `faint`** (paint the hint string yourself in `faint` when empty — egui doesn't do placeholders natively).
- Inner horizontal padding ~9 px.

### 4b. Focused field (the URL bar in the mock)
- Same fill, but **stroke = 1.5px `accent`** (this is the entire focus treatment — no glow, no shadow).
- A blinking text caret in `accent` — egui draws this natively when the field has focus.
- egui: set `selection.stroke` / the active widget `bg_stroke` to the accent at 1.5px width for the focused `TextEdit`.

### 4c. Selection highlight inside text
- Selected text background = `sel` (accent @ 15%). egui: `visuals.selection.bg_fill`.

---

## 5. Dropdown / combo (the GET method selector)
- Looks like a standard field/button: fill = `bg2`, 1px `stroke`, radius 4, height 36, padding ~11.
- Content: value text (monospace, weight 600) on the left, a small caret glyph (`▼`, ~8px, `dim`) on the right.
- Hover: stroke = `accent` (value text unchanged — method is **not** color-coded).
- Open popup menu: see Menus below.
- egui: `ComboBox` with the field's resting visuals; restyle the popup to match menus.

---

## 6. Menus & menu items (combo popup, context menus)
- Popup container: fill = `bg1` (or `bg0`), 1px `stroke` border, radius 5, **minimal/zero shadow**.
- Item: full-width row, radius 4, padding ~6×9, text = `text`.
  - Hovered/highlighted item: fill = `hover` (or `sel` for the currently-chosen value), text stays `text` (or `accent` for the selected value).
- Separators inside menus: 1px line in `stroke-soft`.
- egui: `menu_button` / combo popup; set `widgets.hovered.weak_bg_fill = hover`, popup `window_fill`, popup `window_stroke = (1px, stroke)`, and drop `popup_shadow` to ~0.

---

## 7. List rows / tree items (file browser)
- Row height 24–25, full-width, radius 0 (or 4 if you prefer rounded hover), padding-left scales with depth.
- Resting: fill none, text = `text`; folder disclosure triangle + file `{ }` badge in `faint`.
- Hovered: fill = `hover`.
- **Selected** (current file): left **2px `accent`** bar, row fill = `sel`, badge + label in `accent`, label weight 500.
- egui: draw each row as an interactive `Rect`; on hover paint `hover`, on selected paint `sel` + a 2px accent rect on the left edge, and color the label text accordingly.

---

## 8. Panels, containers & separators
- **Window chrome (title/status bars):** fill = `bg0`.
- **Panels (toolbar, main area, sidebar body):** fill = `bg1`; sidebar uses `side`.
- **Content boxes (code/headers/JSON):** fill = `bg2`, 1px `stroke`, radius 5, inner padding ~9×11.
- **Borders between regions:** a single 1px line in `stroke` (structural) or `stroke-soft` (subtle, within a region). egui: `Separator` or a painted 1px line; for region edges use the panel frame `stroke`.
- No nested borders stacking — one hairline per boundary.

---

## 9. Chips & badges (status pill, timing/size, accent ticks)
- **Status pill (`200 OK`):** fill = `ok-soft` (success @ ~17% alpha), text = `ok`, a 6px `ok` dot, radius 4, padding 1×8, monospace 11.
- **Outline chips (`6.9 ms`, `623 B`):** fill none, 1px `stroke`, text = `dim`, radius 4, padding 1×7.
- **Accent tick** (before section labels): a 2px-wide, 11px-tall `accent` bar, radius 1.
- All small shapes — paint as filled/stroked rounded rects.

---

## 10. Scroll areas & scrollbars
- Content scrolls inside the content boxes. Scrollbar: **thin**, thumb = `scroll`, track transparent, rounded.
- egui: `ScrollArea` with `scroll_bar_width` small (~6–8) and the thumb color set to `scroll` via `widgets.*.bg_fill` on the scroll handle, track left transparent.

---

## 11. Code / JSON rendering (the response body)
This is text painting, not a widget — build a single styled text layout:
- **Line-number gutter:** a fixed left column (~30 px), right-aligned numbers in `gutter` color, non-selectable.
- **Tokens:** color each run by type — key / string / number / bool-null / punctuation (values in README per theme). In egui, assemble a `text::LayoutJob`, pushing each token as a run with its own `TextFormat { color }`, then paint with the monospace font. Long string values wrap; gutter stays aligned per logical line.
- The tokenizer (`walk`) in `CurluApp.dc.html`'s logic is portable pseudocode — reuse its structure.

---

## 12. Typography roles (apply by token, not by element)
| Role | Size | Weight | Color | Font |
|---|---|---|---|---|
| Code / JSON / URL / headers | 12.5 | 400–500 | per token / `text` | **Monospace (JetBrains Mono)** |
| Section label | 10.5 | 600, +0.7 tracking, UPPERCASE | `dim` | UI sans |
| UI text (buttons, rows) | 12.5–13 | 400–500 | `text` | UI sans |
| Chips / footnotes / status | 11 | 400–600 | `dim` / `ok` | Monospace |
| Carets / captions | 9–10 | 600 | `faint` / `dim` | mixed |

Load JetBrains Mono as a custom font and bind it to the toolkit's Monospace family; keep the toolkit's default proportional font for UI sans.

---

## 13. Theme & accent switching
- A **theme** = one full token set (light or dark). Swapping it should reassign the global visual sets in one place (`set_visuals`) — every widget re-resolves automatically next frame.
- An **accent** = one `{ base, hover, on-accent-text }` triple plus the derived `sel` (15% alpha) and `ok-soft` is independent of accent. Store the accent as a single color in app state; reference it for the Send button, focus stroke, selection, accent ticks, and selected-row treatment.
- Persist both choices. The three shipped accents are **Blue (default), Green, Amber**.

---

### One-line summary
Drive everything from tokens; give each widget a per-state `{fill, stroke, text, radius}` bundle; keep strokes hairline, corners 4px, fills solid, and reserve the accent color for the primary action and "active/selected" signals. No gradients, no panel shadows.
