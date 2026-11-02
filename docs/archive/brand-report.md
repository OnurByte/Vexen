> **Historical document, kept for provenance.** It records a plan, a milestone or a snapshot from earlier in the project and may no longer match the code. For the current state see the [status and roadmap](../roadmap.md).

# Vexen brand and repository package

## Positioning

**Give your next coding agent a head start.** Vexen is under development as local, evidence-backed repository memory for coding agents. The README says what exists today and calls out the missing CLI, MCP server, client installer, and cross-session workflow. It labels the sample reminder fictional and the source-install command planned.

The public story should stay evidence-led. The ten-repository Task 1 evaluation found thresholded co-change below plain historical co-change Hit@3 on every repository. That feature remains research-only; the result is a file-retrieval proxy and does not establish agent-task improvement. The README links the full benchmark report and reproducer.

## Deliverables

- `README.md`: focused English overview, accessible diagram alt text, status, available research commands, privacy boundaries, planned install command, and benchmark caveat.
- `brand.md`: voice, palette, type, mark and claims guidance.
- `icon.svg`: canonical four-contour V mark (near-white and Oxocarbon blue). `icon-light.svg` is its light-background variant; `docs/assets/banner.svg` and the hero embed it.
- `docs/assets/hero.svg`: schematic repo-to-evidence-to-next-session illustration.
- `docs/assets/workflow.svg`: failed attempt → local evidence → qualified reminder sequence.
- `docs/assets/example.svg`: a fictional reminder card labeled “ILLUSTRATIVE · FICTIONAL,” with sample source, test, and commit fields and an explicit current-code qualification.

All visible diagram lettering is outlined from Geist / Geist Mono so the SVGs render without a local font installation. The SVGs retain accessible `<title>`/`<desc>` metadata and the README provides meaningful alt text. The assets use vector geometry rather than rendered interface captures; no product screenshot or measured result is implied.

## Provenance

- **Geist / Geist Mono:** requested type direction from [Vercel's Geist page](https://vercel.com/font); source repository and SIL Open Font License notices are at [vercel/geist-font](https://github.com/vercel/geist-font). The official regular, semibold, and mono OTF files and OFL text are included under `docs/assets/fonts/`; the SVG text was outlined with FontTools. Vexen is not a Vercel product and no endorsement is implied.
- **Oxocarbon:** requested palette inspiration from [nyoom-engineering/oxocarbon](https://github.com/nyoom-engineering/oxocarbon). The selected hex values follow the user-provided brief. The logo, diagrams, and composition are original Vexen assets, not copied theme artwork.
- **Product claims:** repository `docs/specification.md`, `docs/benchmarks.md`, task reports, and `docs/progress.md` as present at package preparation. Claims are limited to the current source and recorded evaluation; plans are labeled as plans.

## Search presentation

Suggested GitHub repository description: **Local, evidence-backed memory for coding agents working in the same repository.**

Suggested repository topics (all lowercase):

`coding-agents`, `agent-memory`, `mcp`, `rust`, `local-first`, `developer-tools`, `git`, `sqlite`, `open-source`, `software-engineering`

Suggested SEO title: **Vexen — Local Repository Memory for Coding Agents**

Suggested meta description: **Vexen is an open-source Rust project exploring local, evidence-backed repository memory for coding agents, with clear provenance and honest evaluation.**

Relevant search phrases for future docs and release notes, when the described feature actually exists: local coding-agent memory; repository memory for AI coding agents; MCP server for coding agents; Git history change-impact analysis; evidence-backed failed-attempt memory; private, offline coding-agent tools. Avoid saying “best,” “proven,” “prevents bugs,” or “improves coding success” without supporting comparative evidence.

## Current limitations

The repository has no ready-to-install CLI or MCP integration yet. No client smoke-test coverage, released binary, stable compatibility matrix, measured coding-agent outcome uplift, or production memory workflow is claimed here. The planned `cargo install --git https://github.com/OnurByte/Vexen --locked` line in the README must remain clearly marked as planned until the binary and public repository are real and independently checked.
