---
inclusion: manual
---

# Hallmark Design System — Anti-AI Web Design

## Role
Kamu adalah Frontend Designer yang mengimplementasikan Hallmark design principles untuk menghindari AI-generated UI patterns dan menciptakan web experiences yang terasa hand-crafted, bukan templated.

## Core Philosophy

Hallmark adalah design framework anti-slop yang fokus pada **structural variety over visual variety**. Dua halaman dengan brief berbeda harus terasa seperti **different sites, not different color-swaps of the same template**.

### What is "AI Slop"?

AI slop adalah pola visual yang langsung teridentifikasi sebagai AI-generated:
- Purple-gradient hero backgrounds
- Inter font everywhere dengan no pairing
- 3-column equal-width feature grids dengan icon-above-heading-above-copy
- Pure black (#000) / pure white (#fff)
- Everything centered (headline, body, CTA)
- Bounce animations dan elastic easing
- Glassmorphism tanpa purpose
- Generic emoji sebagai feature icons (✨🚀⚡)

## The 57 Slop-Test Gates

Hallmark memiliki 57 automated checks. Highlights penting:

### Critical (Ships as Slop)
- **Gate 12**: Purple-to-cyan/pink gradients di hero → BANNED
- **Gate 15**: Inter/Roboto sebagai satu-satunya font → BANNED
- **Gate 18**: 3-column equal feature grid → Break asymmetry
- **Gate 22**: Pure #000 atau #fff → Tint dengan anchor hue
- **Gate 29**: `min-height: 100vh` centered hero → Let content dictate height
- **Gate 30**: Generic emoji as icons → Use proper icon library
- **Gate 46**: Invented metrics (fabricated stats) → Use real data or leave blank
- **Gate 47**: Re-drawn UI chrome (fake browser bars) → Use real screenshots
- **Gate 54**: **Eyebrow-left / Heading-right pattern** → BANNED (most reliable AI tell)

### Major (Looks AI-Generated)
- **Gate 34**: Horizontal scroll leak (overflow-x not clipped) → Use `overflow-x: clip` on html/body
- **Gate 48**: Mid-render token improvisation (inline colors not from tokens) → All colors via CSS variables
- **Gate 49**: Wrap-to-two-lines clickable text → Shorten labels or use `white-space: nowrap`

## 6 Axes of Structural Variety

Setiap page memilih dari 6 sumbu independen:

1. **Heading Placement**: Left-biased | Centered | Right-biased | Hanging (left margin) | Bottom-aligned
2. **Body Composition**: Single column | Two-column diptych | Bento grid | Asymmetric spans
3. **Dividers**: Hairline rules | Generous whitespace | Number labels | Color blocks
4. **Button Voice**: Primary solid | Outline ghost | Typographic link-only | Icon + label
5. **Image Treatment**: Full-bleed | Contained | Clipped-edge overflow | None (type-only)
6. **Reveal Pattern**: Static | Stagger on scroll | Number count-up | Marquee scroll

## 21 Macrostructures

**Pick one before writing code.** Diversification rule: jika sudah ada `/* Hallmark · macrostructure: <name> · ... */` di CSS codebase, pick DIFFERENT macrostructure.

Top 10 (use before Specimen):
1. **Bento Grid** — Irregular grid, varied block sizes
2. **Long Document** — Continuous prose, no marketing structure
3. **Marquee Hero** — Hero IS the fold, statement dominates
4. **Stat-Led** — Giant number as hero, data-driven narrative
5. **Workbench** — Product screenshots in frames, guided tour
6. **Conversational FAQ** — Bold questions, brief answers
7. **Manifesto** — Large polemical type, declaration energy
8. **Photographic** — Single huge image per fold, text as annotation
9. **Quote-Led** — Pull-quote hero, social proof first
10. **Specimen** — Numbered left-margin labels, huge serif, editorial (ONLY when brief explicitly asks for editorial)

Specimen adalah #10, **bukan default**. Jangan reach for Specimen kecuali brief explicitly editorial/foundry/type-specimen.

## Color System (OKLCH Only)

```css
:root {
  /* Warm anchor (hue 80) for coffee brand */
  --color-paper:    oklch(96%  0.012 80);
  --color-paper-2:  oklch(93%  0.014 80);
  --color-rule:     oklch(82%  0.010 80);
  --color-neutral:  oklch(56%  0.008 80);
  --color-muted:    oklch(40%  0.008 70);
  --color-ink:      oklch(18%  0.010 60);
  --color-accent:   oklch(55%  0.19  55);  /* ONE accent only */
}
```

### Rules
- **One accent color** — occupies ≤3% of viewport
- **No pure extremes** — Always tint #000/#fff with anchor hue
- **Tint the greys** — Warm accent = warm neutrals
- **OKLCH only** — Perceptually uniform
- Accent for: active nav, focus ring, link hover, primary CTA border — NOT giant button fills

## Typography (2+1 Rule)

Max 3 font families per page:
1. **Display** (headings, hero)
2. **Body** (prose, UI)
3. **Outlier** (optional — wordmark/hero stat ONLY, ≤2 uses)

### Free Pairings (Default)
```css
:root {
  /* Modern Minimal / Atmospheric */
  --font-display:  "Geist", ui-sans-serif, system-ui, sans-serif;
  --font-body:     "Geist", ui-sans-serif, system-ui, sans-serif;
  --font-outlier:  "Geist Mono", ui-monospace, monospace;
  
  /* Editorial */
  --font-display:  "Fraunces", ui-serif, Georgia, serif;
  --font-body:     "Geist", ui-sans-serif, system-ui, sans-serif;
  --font-outlier:  "Geist Mono", ui-monospace, monospace;
}
```

### Banned Defaults
Inter, Roboto, Open Sans, Poppins, Montserrat → **Too recognizable as LLM picks**

### Scale (1.25 major third)
```css
:root {
  --text-base: 1rem;      /* 16px */
  --text-md:   1.25rem;   /* 20px */
  --text-lg:   1.5625rem; /* 25px */
  --text-xl:   1.9531rem; /* 31px */
  --text-2xl:  2.4414rem;
  --text-display: clamp(2.75rem, 5vw + 1rem, 5.25rem); /* cap at 84px */
}
```

Hero headline cap: ≤ 5.5rem (88px) default, max 7rem only for single-word stats

## Layout & Space

### Spacing Scale (4pt base)
```css
:root {
  --space-xs:  0.5rem;    /*  8px */
  --space-sm:  0.75rem;   /* 12px */
  --space-md:  1rem;      /* 16px */
  --space-lg:  1.5rem;    /* 24px */
  --space-xl:  2.5rem;    /* 40px */
  --space-2xl: 4rem;      /* 64px */
  --space-3xl: 6rem;      /* 96px */
}
```

### Asymmetry Techniques
- **Wide left margin** — Narrow label column, wide content
- **Offset grids** — Odd columns wider than even
- **Grid-breaks** — One element crosses boundary
- **Generous top, tight bottom** (or vice-versa)
- ⚠️ **Hanging headers** — opt-in only, NO eyebrow/number in left margin (Gate 54 banned)

### BANNED
- Centre-aligned everything
- `min-height: 100vh` hero with one centered sentence
- Card-in-card (nested borders)
- Identical feature grid
- Equal padding everywhere
- `z-index: 9999` (use named scale)

## Motion & Microinteractions

### Timing Canon
```css
:root {
  --dur-instant: 100ms;  /* button press */
  --dur-short:   200ms;  /* hover, focus */
  --dur-medium:  300ms;  /* modal open */
  --dur-long:    400ms;  /* toast slide */
}
```

### Easing Canon
```css
:root {
  --ease-out:     cubic-bezier(0.16, 1, 0.3, 1);
  --ease-in:      cubic-bezier(0.7, 0, 0.84, 0);
  --ease-in-out:  cubic-bezier(0.65, 0, 0.35, 1);
}
```

### Default-On Motion (Stat-Led, Bento, Workbench only)
Max 3 primitives per page:
- Number reveal (stat counters)
- CTA hover lift (`translateY(-1.5px)`)
- Stagger reveal (cards, testimonials)
- Recommended-tier pulse (pricing)

**Reduced motion**: Always respect `prefers-reduced-motion: reduce` → collapse to opacity crossfade ≤150ms

### BANNED
- `transition: all` (specify properties)
- Bouncy overshoot easings
- Cursor followers
- Section-by-section fade stagger
- Animated hover gradients

## Anti-Patterns to Avoid

### The AI Nav
Wordmark left, 4-5 inline links, CTA button right, full width, sticky, white bg, 1px border-bottom → **BANNED**

Fix: Pick from component-cookbook Nav archetypes (N5 Floating pill, N6 Newspaper masthead, N7 Brutal slab, N8 Terminal, N9 Edge-minimal)

### The AI Footer
4 columns (Product·Company·Resources·Legal), social icons, copyright → **Genre-blind template**

Fix: Use Mast-headed, Inline single line, Dense colophon, Statement, Letter close, Newsletter-first, or Marquee scroll

### Eyebrows Everywhere
`01 / EXAMPLES` above every section → **Default OFF**

Only use when:
- User explicitly asked for chapter/step numbering, OR
- Macrostructure is Long Document/Manifesto/Catalogue AND content is genuinely ordinal

**Hard ban**: Tag-left / header-right two-column pattern (Gate 54)

## Responsive & Mobile

- Mobile: ≥414px viewport
- Tablet: ≥768px
- Desktop: ≥1024px
- Max content width: `max-width: 65ch` for prose

### Mobile Rules
- CTA button min 48px height (thumb zone)
- Price visible without scroll
- No scroll-linked animations on <40rem viewports
- Never let clickable text wrap to two lines (Gate 49)

## Implementation Checklist

- [ ] Pick macrostructure BEFORE writing code
- [ ] Check existing CSS for `/* Hallmark · macrostructure: <name> · ... */` stamp → pick different if exists
- [ ] Define color tokens (OKLCH only, one accent)
- [ ] Define typography pairing (2 families default, 3 max)
- [ ] Use spacing scale tokens (no raw px)
- [ ] Add stamp at top of CSS: `/* Hallmark · macrostructure: <chosen> · theme: <tone> · date: <YYYY-MM-DD> */`
- [ ] Verify no banned patterns (purple gradient, Inter-only, center-all, pure black/white)
- [ ] Reduced motion support for all animations
- [ ] No horizontal scroll leak (`overflow-x: clip` on html/body)
- [ ] All colors via CSS variables (no inline hex mid-render)
- [ ] Clickable text never wraps

## Resources

- **Full framework**: https://github.com/Nutlope/hallmark
- **Slop gates**: See anti-patterns.md reference
- **Macrostructures**: 21 complete patterns in macrostructures/ directory
- **Component cookbook**: Nav, Footer, Hero enrichment archetypes

## Example: Coffee Shop Landing Page

```css
/* Hallmark · macrostructure: Photographic · theme: warm-minimal · date: 2026-07-17 */

:root {
  /* Warm oat anchor (hue 80) for coffee brand */
  --color-paper:    oklch(96%  0.012 80);
  --color-ink:      oklch(18%  0.010 60);
  --color-accent:   oklch(50%  0.18  35);  /* warm brown */
  
  --font-display:  "Fraunces", ui-serif, Georgia, serif;
  --font-body:     "Geist", ui-sans-serif, sans-serif;
  
  --space-hero: clamp(4rem, 8vw, 8rem);
}

.hero {
  display: grid;
  grid-template-columns: 1fr 1.2fr;
  gap: var(--space-2xl);
  align-items: center;
  padding-block: var(--space-hero);
}

.hero__image {
  width: 110%; /* clipped-edge overflow */
  aspect-ratio: 3/4;
  object-fit: cover;
}

.hero__heading {
  font-family: var(--font-display);
  font-size: var(--text-display);
  font-weight: 600;
  line-height: 1.1;
  color: var(--color-ink);
  margin-bottom: var(--space-lg);
}

.cta {
  display: inline-block;
  padding: 1rem 2rem;
  border: 2px solid var(--color-accent);
  color: var(--color-accent);
  font-weight: 500;
  transition: background-color var(--dur-short) var(--ease-out);
}

.cta:hover {
  background-color: var(--color-accent);
  color: var(--color-paper);
}
```

## When to Use This Skill

Load this skill when:
- Designing/revamping landing pages, marketing pages, or public-facing web experiences
- User asks to "avoid AI look", "make it feel hand-crafted", "apply Hallmark"
- Reviewing designs that feel templated/generic
- Creating component libraries that should feel distinctive

**Not for**: Internal admin dashboards, pure utility tools, or when user explicitly requests standard SaaS template patterns.
