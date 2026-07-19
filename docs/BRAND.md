# L.ai — Brand System

> Umbrella mark for the offline, deterministic, fail-loud verification ecosystem.
> Public face: **L.ai**. Code names (Laverna, CID, CID-Bridge, Tanto, Athena) are internal.

This document is the single source of truth for how the **L.ai** ecosystem
presents itself across repositories, READMEs, docs, and project assets. Every
repository in the `nutypebuddha` org that belongs to the ecosystem copies this
file verbatim and applies the same logo, header, and palette.

---

## The mark

```
        ___
       /   \     Ł  ·  a i
      |  ▲  |
      | ▛▀▀▛|
       \___/
```

The **Ł** is an old-English capital L (crossed descender) rendered as a
glitching cyberpunk witch-hat glyph: a sharp conical crown over a flickering
terminal block. It signals "the L that thinks" — logic under a hat, the
assistant that never guesses.

- Primary vector asset: [`docs/brand-LAI.svg`](brand-LAI.svg) (witch-hat Ł glyph)
- Ecosystem map: [`docs/ecosystem.svg`](ecosystem.svg)

## Palette

Restrained by design. The base is the original cyberpunk cyan/purple; the
**rust-orange accent** was added to tie the brand to Rust and to warm the
otherwise cold palette. Use rust-orange *sparingly* — for one or two accents per
page (a rule, a key badge, a single callout). Everything else stays black /
deep-purple / silver.

| Token | Hex | Use |
|-------|-----|-----|
| `--ink` (black) | `#0a0a12` | Background, terminal blocks |
| `--deep-purple` | `#b026ff` | Primary brand fill, headers, hat crown |
| `--cyan` | `#7df9ff` | Secondary glow, outlines, mono text |
| `--magenta` | `#ff2bd6` | Glitch slices, hover accents |
| `--silver` | `#c7c9d1` | Body text on dark, secondary lines |
| `--rust` (accent) | `#c2410c` | **Primary accent** — key badges, rules, CTAs |
| `--rust-bright` | `#ea580c` | Rust accent, hover/active state |

Contrast rule: body text is `--silver` on `--ink`. Headings are `--cyan` or
`--deep-purple`. Never put `--rust` on a light background at small sizes.

## Voice

- **Name the function, not the thing.** Every artifact collapses to the pure
  function it performs. Describe *what the function does*, not a mascot.
- **Verify, don't trust.** The tagline is a contract: deterministic, fail-loud,
  no fabrication, no network at runtime.
- Confident, technical, minimal. No hype words. Benchmarks over adjectives.

| Code name | Pure function | L.ai label |
|-----------|---------------|------------|
| Laverna (engine) | `deterministic_proof(input) -> verified_object` | L.ai · Proof |
| CID | `validate_token(stream) -> gated_verdict` | L.ai · Gate |
| CID-Bridge | `bridge(chatbot, [Gate, Proof]) -> merged_verdict` | L.ai · Bridge |
| Tanto (math) | `compute(expr) -> value` | L.ai · Compute |
| Athena | `reason(query) -> validated_chain` | L.ai · Athena |
| Companion | `assist(query) -> classified_receipt` | L.ai · Assist |

## Repository header convention

Every README opens with the same branded block, then `L.ai · <Function>`:

```markdown
<div align="center">

# Ł L.ai · *Proof*

**Offline, deterministic verification for AI. Verify, don't trust.**

</div>
```

- README headers: `L.ai · <Function>` with the code name as a subtitle.
- All ecosystem repositories share this file verbatim.
- Consistent: README layout, logo, header images, color palette, LICENSE,
  CONTRIBUTING.

## The ecosystem map

```
        Ł  (creator identity / umbrella)
            │
     ┌──────┴──────┐
     │             │
  Athena         Mana Core
     │             │
     ├─────────────┤
           LAI  (offline verification substrate: Proof · Gate · Bridge · Compute)
```

Projects that belong to the ecosystem carry the witch-hat Ł and the palette so
a visitor opening any one immediately recognizes the family.

## License & contribution

- Source: Apache-2.0 (see `LICENSE`, `NOTICE`) unless a crate states otherwise.
- Each repo ships `LICENSE`, `NOTICE`, and `CONTRIBUTING.md`.
- The verification contract is unchanged across the family: pure functions
  only, no global state, deterministic output, fail loud.
