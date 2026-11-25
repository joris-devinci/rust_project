---
name: Lending simulation
description: Debt is water rising toward a mark it must not pass.
colors:
  river: "#0e3f4f"
  river-deep: "#0a303d"
  water: "#2f86a6"
  amber: "#f0b400"
  amber-pale: "#fbefc8"
  limit: "#b92c26"
  limit-light: "#ff6a5e"
  mist: "#cfe0de"
  stone: "#e6ecea"
  stone-dark: "#d3deda"
  panel: "#f7f9f8"
  white: "#ffffff"
  ink: "#0e2f3a"
  ink-soft: "#3f5d66"
typography:
  display:
    fontFamily: "Barlow, Helvetica, sans-serif"
    fontSize: "6rem"
    fontWeight: 700
    lineHeight: 1
    letterSpacing: "-0.025em"
  headline:
    fontFamily: "Barlow, Helvetica, sans-serif"
    fontSize: "1.875rem"
    fontWeight: 700
    lineHeight: 1.2
  figure:
    fontFamily: "Barlow, Helvetica, sans-serif"
    fontSize: "1.875rem"
    fontWeight: 600
    lineHeight: 1.2
  title:
    fontFamily: "Barlow, Helvetica, sans-serif"
    fontSize: "1.125rem"
    fontWeight: 600
    lineHeight: 1.556
  body:
    fontFamily: "Barlow, Helvetica, sans-serif"
    fontSize: "1rem"
    fontWeight: 400
    lineHeight: 1.5
  label:
    fontFamily: "Barlow, Helvetica, sans-serif"
    fontSize: "1rem"
    fontWeight: 600
    lineHeight: 1.5
  small:
    fontFamily: "Barlow, Helvetica, sans-serif"
    fontSize: "0.875rem"
    fontWeight: 400
    lineHeight: 1.429
rounded:
  sm: "4px"
  md: "6px"
  lg: "8px"
spacing:
  gutter: "16px"
  gutter-wide: "32px"
  panel: "20px"
  panel-wide: "24px"
  hero-wide: "28px"
  gap: "24px"
  column-gap: "32px"
components:
  button-primary:
    backgroundColor: "{colors.river}"
    textColor: "{colors.white}"
    typography: "{typography.label}"
    rounded: "{rounded.md}"
    padding: "12px 16px"
    width: "100%"
  button-primary-hover:
    backgroundColor: "{colors.river-deep}"
  button-secondary:
    backgroundColor: "{colors.stone}"
    textColor: "{colors.ink}"
    typography: "{typography.label}"
    rounded: "{rounded.md}"
    padding: "0 16px"
  button-secondary-hover:
    backgroundColor: "{colors.stone-dark}"
  button-on-river:
    backgroundColor: "{colors.white}"
    textColor: "{colors.river}"
    typography: "{typography.label}"
    rounded: "{rounded.md}"
    padding: "12px 20px"
  button-on-river-hover:
    backgroundColor: "{colors.stone}"
  button-header:
    backgroundColor: "rgb(255 255 255 / 0.15)"
    textColor: "{colors.white}"
    typography: "{typography.label}"
    rounded: "{rounded.md}"
    padding: "6px 12px"
  button-header-hover:
    backgroundColor: "rgb(255 255 255 / 0.25)"
  segment:
    textColor: "{colors.ink}"
    typography: "{typography.label}"
    rounded: "{rounded.sm}"
    padding: "8px 12px"
  segment-active:
    backgroundColor: "{colors.river}"
    textColor: "{colors.white}"
  field:
    backgroundColor: "{colors.white}"
    textColor: "{colors.ink}"
    typography: "{typography.body}"
    rounded: "{rounded.md}"
    padding: "10px 12px"
  panel:
    backgroundColor: "{colors.panel}"
    textColor: "{colors.ink}"
    rounded: "{rounded.lg}"
    padding: "{spacing.panel-wide}"
  hero:
    backgroundColor: "{colors.river}"
    textColor: "{colors.white}"
    rounded: "{rounded.lg}"
    padding: "{spacing.hero-wide}"
  notice:
    backgroundColor: "{colors.amber-pale}"
    textColor: "{colors.ink}"
    rounded: "{rounded.lg}"
    padding: "16px 20px"
  badge:
    textColor: "{colors.white}"
    typography: "{typography.title}"
    rounded: "{rounded.sm}"
    padding: "2px 10px"
  badge-unsafe:
    textColor: "{colors.amber}"
  badge-warning:
    backgroundColor: "{colors.amber}"
    textColor: "{colors.ink}"
  badge-liquidatable:
    backgroundColor: "{colors.limit}"
    textColor: "{colors.white}"
  liquidation-report:
    backgroundColor: "{colors.limit}"
    textColor: "{colors.white}"
    rounded: "{rounded.lg}"
    padding: "{spacing.hero-wide}"
---

# Design System: Lending simulation

## Overview

**Creative North Star: "Flood Marks"**

A flood gauge beside a river. Debt is the water, borrowing power is the graduated staff, and 100% is the mark it must not pass. One dark river panel carries the level; everything else is quiet, pale and flat around it.

Tokens live in the `@theme` block of `app/globals.css`. Values without a token (type sizes, radii, spacing) are Tailwind 4 defaults.

**Key Characteristics:**
- Dark river panel on pale stone ground
- One family, Barlow, in three weights, with tabular figures
- Flat surfaces, no shadows
- Red means the limit, and nothing else
- Every color state is also written in words

## Colors

Cold river tones, one warm warning color, one reserved red.

### Primary
- **River** (#0e3f4f): header, level panel, primary button, selected action.
- **Deep River** (#0a303d): chart caption band and primary button hover.

### Secondary
- **Water** (#2f86a6): the water fill, focus rings on light surfaces, caret.

### Tertiary
- **Amber** (#f0b400): the 90% mark, the Unsafe badge outline, the Warning badge fill.
- **Pale Amber** (#fbefc8): notices and refused actions, always with ink text.
- **Limit Red** (#b92c26): the Liquidatable badge and the liquidation report.
- **Light Limit Red** (#ff6a5e): the 100% line, in the gauge and the app icon. It is the same red, lightened to read on river.

### Neutral
- **Stone** (#e6ecea): page ground, secondary button, action switch track.
- **Dark Stone** (#d3deda): row dividers and light hovers.
- **Panel** (#f7f9f8): light panels.
- **White** (#ffffff): fields, text on river and on red, buttons on both.
- **Mist** (#cfe0de): gauge staff, graduations, quiet text on river.
- **Ink** (#0e2f3a): text.
- **Soft Ink** (#3f5d66): secondary text.

### Named Rules
**The Reserved Red Rule.** Both reds belong to the 100% mark and to liquidation. A form error, a lost account or a stale price is amber.

**The Word First Rule.** Color never carries a state alone: each badge shows its name, and each gauge mark is labelled.

## Typography

**Font:** Barlow (with Helvetica, sans-serif), in 400, 600 and 700, for everything.

**Character:** A plain signage face. Size and weight make the hierarchy; text stays in sentence case.

### Hierarchy
- **Display** (700, 96px, 60px below 640px, line-height 1, -0.025em): the utilization only.
- **Headline** (700, 30px): welcome question (36px from 640px) and liquidation title.
- **Figure** (600, 30px, 24px below 640px): the margin in dollars.
- **Title** (600, 18px): panel headings and status badges. The product name in the header is slightly larger (700, 20px).
- **Body** (400, 16px, line-height 1.5): text and table values. Long text is capped at 65ch.
- **Label** (600, 16px): buttons, field labels, row headers, totals.
- **Small** (400, 14px): gauge graduations, chart caption, dollar values under quantities. Marks and caption warnings use 600.

### Named Rules
**The Tabular Rule.** Figures are tabular on the whole page, so a price tick never shifts the layout.

## Layout

A centered container (1280px) with a 16px gutter, 32px from 640px. From 1024px, two columns at 2:1 with a 32px gap: level then position on the left, actions then prices on the right. Below, one column in the order level, actions, position, prices. Blocks are 24px apart.

Padding steps up at 640px: light panels 20px to 24px, river and red panels 20px to 28px. The gauge is 240px tall, then 288px, and the water runs to the panel's right edge.

## Elevation & Depth

No shadows. Depth is tonal: stone ground, lighter panels, dark river on top. Rows are separated by 1px dividers.

## Shapes

Gentle corners, by size of the object: panels and notices 8px, buttons, fields and form errors 6px, badges and switch segments 4px. The river panel clips its content, so the water and the caption band follow its corners. The water has square steps: each observed level holds until the next one.

## Components

### Buttons
- **Primary:** river with white text, full width of the form. Hover is deep river.
- **Secondary:** stone with ink text (Max). Hover is dark stone.
- **On river or red:** white, with text in the color of the panel behind it. Hover is stone.
- **Header:** translucent white (15%, 25% on hover).
- **States:** colors transition in 150ms; disabled is 60% opacity; the label says what is happening ("Sending", "Connecting").
- **Focus:** 3px water outline, 2px offset; white on river and on red.

### Action switch
Three equal segments in a stone track. The selected one is river with white text.

### Status badge
A 2px border and a word. Emphasis rises with health: mist outline (No debt at 60%, Safe), amber outline (Unsafe), amber fill (Warning), limit red fill with a white border (Liquidatable).

### Panels
Light panel with a title and content, no border. Tables and price rows use dark stone dividers.

### Fields
White, 6px corners, thin soft ink border, semibold label above. The field keeps its look on error: the message appears below in pale amber and says what to do.

### Notices
Pale amber block above the content for a lost account, an error or stale values.

### Gauge (signature)
A mist staff with graduations on the left, the marks drawn across the water. The marks get heavier: 75% is 1px mist, 90% is 2px amber, 100% is 4px light limit red. Each has a 2px deep river edge, so it stays readable over the water. The water is a water fill under a 2px white line. When the scale tightens, the caption says "Zoomed in from"; the caption also gives the time of the first and last observation.

### Liquidation report (signature)
A full-width limit red panel above the columns, kept until dismissed, with a before and after table. It rises from below in 700ms (`cubic-bezier(0.16, 1, 0.3, 1)`), the only keyframe animation; it is off when reduced motion is requested.

## Do's and Don'ts

### Do:
- **Do** use the tokens of `@theme` for every color.
- **Do** write the state in words next to its color.
- **Do** say when a value is observed, stale, frozen or zoomed.
- **Do** keep one display-size figure: the utilization.

### Don't:
- **Don't** use either red outside the 100% mark and liquidation.
- **Don't** add shadows: use the tonal steps.
- **Don't** smooth the water between two observations.
