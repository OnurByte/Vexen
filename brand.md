# Vexen brand

## Promise

**Give your next coding agent a head start.** Vexen is being built to carry relevant, evidence-backed repository context between coding sessions, locally and across agent harnesses.

## Voice

Write like a thoughtful teammate. Lead with the practical use, explain evidence in ordinary language, and state uncertainty plainly. Prefer “what happened,” “where it came from,” and “when it may no longer apply” to claims of intelligence or certainty. Say that no relevant evidence is a useful result. Label plans, examples, and measured findings distinctly.

Avoid promises that Vexen prevents bugs, improves agent success, knows what files must change, or has broad client compatibility. Do not suggest that Vexen records transcripts, runs remembered commands, gates user decisions, or has a hosted service. Say when a feature is planned or research-only.

## Visual system

- **Canvas:** Oxocarbon-inspired near-black `#161616` and raised charcoal `#262626`.
- **Text:** warm near-white `#f2f4f8`.
- **Signal:** Oxocarbon blue `#78a9ff` (the mark's blue); use it for evidence and active paths.
- **Accent:** light blue `#a6c8ff`; use sparingly for a stored note, qualification, or secondary signal.
- Keep contrast high and layouts calm. Use generous spacing, simple shapes, and labeled explanations in nearby page text. Avoid dashboards, glowing gradients as decoration, and fake product screenshots.

The mark is [`icon.svg`](icon.svg): four contours that converge into a forward-pointing V, in near-white `#f2f4f8` and Oxocarbon blue `#78a9ff` on a transparent background. It is the single logo for the README, banners, docs, and package pages; do not redraw or recolor it ad hoc. Do not use Vercel's mark or imply affiliation. The README illustrations are schematic, not application screenshots or measured results.

## Logo files

- [`icon.svg`](icon.svg): canonical mark for dark surfaces (`#161616` and darker).
- [`icon-light.svg`](icon-light.svg): same geometry in `#161616` and `#0f62fe` for light surfaces. The README switches between them with `<picture>`.
- [`docs/assets/banner.svg`](docs/assets/banner.svg): 1280×640 mark, wordmark, and tagline; use it for the GitHub social preview (upload it as a PNG in repository settings).
- [`docs/assets/hero.svg`](docs/assets/hero.svg): README hero; embeds the mark from `icon.svg`.
- [`docs/assets/social-preview.png`](docs/assets/social-preview.png): the banner as a PNG, ready to upload as the GitHub social preview.
- README illustrations (`problem`, `workflow`, `example`, `principles`, `terminal`, `status`): dark `#161616` cards on `#262626`, Geist text outlined to paths so it renders the same everywhere, only the mark's blue and white. `terminal.svg` is real command output; `problem.svg` and `example.svg` are labeled fictional.

Keep clear space around the mark of at least a quarter of its width and do not render it below 24 px wide. Diagrams use only the mark's blue and its lighter tint; no other hues.

## Type

Use [Geist](https://vercel.com/font) for headings and body text, and Geist Mono for commands and identifiers. This package includes the official regular, semibold, and mono OTF files under `docs/assets/fonts/`, together with the repository's SIL Open Font License text. The diagram lettering is outlined from Geist with FontTools so it renders consistently without a local font install; each SVG also has accessible title and description text. Source and license notices are in the [Vercel Geist Font repository](https://github.com/vercel/geist-font). Vexen is independent and is not endorsed by Vercel.

## Message examples

- “A past test found duplicate charging after this retry change. Check whether the same conditions apply.”
- “No relevant evidence found.”
- “This is a planned install command; the CLI is not available yet.”

Not: “Vexen knows what your agent needs,” “Never repeat a mistake,” or “Works with every coding agent.”

## Platform and client icons

README platform and client marks live in `docs/assets/icons/`; the agent panel is `docs/assets/clients.svg`. The original mark geometry is retained in a single Vexen blue (`#78a9ff`) on rounded near-black tiles (`#161616`). The panel uses charcoal cards (`#262626`), near-white labels (`#f2f4f8`) and light-blue supporting text (`#a6c8ff`). Labels are outlined from the bundled Geist fonts, matching the existing illustrations.

Third-party source and license information is recorded in `docs/assets/icons/sources.json`, with license notices in `docs/assets/icons/licenses/`. Lobe Icons 1.101.0 marks are MIT-licensed; the Linux, VS Code and Zed marks come from Simple Icons (CC0); the Continue mark comes from its Apache-2.0 repository. Windows 11 uses a four-pane geometric rendering. Product marks remain the property of their respective owners and identify compatibility surfaces, not endorsement. The panel links to setup and test details because support levels differ between clients.
