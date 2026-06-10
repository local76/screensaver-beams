# beams

> Spotlights sweep across a starfield, lighting up your live OS logo as they pass.

Multiple moving spotlights illuminate dust particles and a centered live OS logo. The logo lights up where the beams hit it.

## Visual elements

- **Spotlights**. Several beams with different angles, speeds, and spreads that sweep back and forth.
- **Dust particles**. Small floating particles that get lit when beams pass over them.
- **Background stars**. Subtle twinkling stars (can be disabled via registry).
- **Centered logo**. The live OS name + kernel (e.g. "WIN11" + kernel line) rendered in the middle. Beams dynamically light up sections of it.

## Dynamic / live behavior

- **Live logo**. The block in the middle uses your actual OS name and kernel pulled at runtime via `get_system_info()`.
- **System load reactions**. Higher CPU/memory pressure increases beam speed, spread, and intensity. The lighting becomes more dramatic under load.
- **Per-machine personality**. `host_bias` (from hostname) subtly shifts beam behavior so different computers feel unique.
- **Accent color**. Beams and lighting are tinted by your current system accent.

## Configuration (registry)

Under `HKEY_CURRENT_USER\Software\local76\beams`:

- `BeamCount`: 2–6 (clamped). Controls how many spotlights are active.
- `TwinkleStars`: 0 = off, 1 = on (background stars).

Global settings (under `...\Settings`):

- `ColorTheme`: 0 = system accent (default), or 1–5 for fixed colors.
- `GlobalScanlines`: 1 = enable scanline effect across all scenes.

## Notes

- The logo is fully dynamic — it reflects whatever OS the screensaver is running on.
- One of the more atmospheric and calm scenes when system load is low.
- Excellent on multi-monitor setups.

Part of the [screensavers](https://github.com/local76/screensavers) collection. See the root README for installation.
