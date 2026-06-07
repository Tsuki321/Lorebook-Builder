# Lorebook Builder

A polished, single-binary desktop app for crawling Fandom/MediaWiki lore wikis
and producing **SillyTavern-compatible world info** JSON files.

Built entirely in Rust with `egui`. Single `.exe`, no installer, no Node,
no web tech.

## Features

- **Flexible crawler** — paste any Fandom/MediaWiki URL, pick categories,
  add custom ones
- **Smart extraction** — pulls character names, aliases, titles, and prose
  from standard infobox templates
- **Live progress** — see pages/second, ETA, errors as you crawl
- **Disk cache + resume** — re-runs are instant, crashes are recoverable
- **In-app editor** — table view + side detail editor, add/remove/edit
- **Polished UI** — Catppuccin theme, dark/light toggle, custom typography

## Building

This repo does not support local builds. The artifact is produced by
**GitHub Actions** and downloaded as an `.exe` from the Actions run.

### Trigger a build

1. Push this repo to GitHub
2. Go to the **Actions** tab
3. Select the **build** workflow on the left
4. Click **Run workflow** → **Run workflow**
5. Wait ~5 minutes
6. Download `wiki-lore-crawler-windows` from the run's Artifacts section

The build runs on `windows-latest` and produces a stripped
`x86_64-pc-windows-msvc` release binary.

## Usage

1. Launch the `.exe`
2. Go to the **Crawl** tab
3. Enter a wiki URL (e.g. `lordofthemysteries.fandom.com`)
4. Check the categories you want (or add custom ones)
5. Click **Start**
6. Switch to the **Library** tab to review/edit entries
7. Go to the **Export** tab and save your `lorebook.json`
8. Drop the file into SillyTavern's lorebook slot

## License

MIT — see [LICENSE](LICENSE).
