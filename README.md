# JKS Premietanie piesní

Toto je aplikácia na správu a premietanie liturgických piesní (JKS a ďalšie typy) v kostole.  
Používa textové rozhranie v termináli na výber a správu piesní a samostatnú GUI aplikáciu na fullscreen premietanie na vybranom monitore.

---

# JKS Song Projection

This repository contains a Rust workspace for managing and projecting liturgical songs (JKS and other types) in church.

There are two main parts:

- a **terminal user interface (TUI)** used to manage the song database, select songs for projection, and start the presentation
- a **presentation GUI** that shows the selected songs in fullscreen on a chosen monitor

## Features

- Import songs from a CSV file into a SQLite database
- Store verses (stanzas) and song types (JKS categories, hymns, antiphons, Taizé, etc.)
- Search and filter songs by text and type
- Build a playlist of songs for projection
- Start a separate fullscreen presenter window on a selected monitor
- Select monitor and save its geometry into `monitor_config.json`
- Save the current playlist into a temporary JSON file read by the presenter
- Native dialogs for entering song IDs and types

## Technology

- **Rust**
- **SQLite + Diesel** for persistence
- **ratatui + tui-tree-widget** for the terminal UI
- **eframe / egui** for the fullscreen presenter GUI
- **inputbox, native-dialog** for native dialogs
- **crossterm** for keyboard input in the TUI

## Dependencies

Rust crates (pulled automatically via Cargo):

- `diesel` + `diesel_migrations` (SQLite backend)
- `ratatui`
- `tui-tree-widget`
- `crossterm`
- `eframe` / `egui`
- `native-dialog`
- `inputbox`
- `serde`, `serde_json`
- `dotenvy`

System / external tools:

- **SQLite** (CLI optional, library is bundled via Diesel)
- A text editor available in `PATH`:
  - on Linux the project currently expects `mousepad` for editing lyrics in a temporary file
  - on Windows the fallback is `notepad`
  - on macOS the system `open` command is used to launch the default editor

## Workspace layout

The project is a Cargo workspace (root `Cargo.toml`) with several crates (names here are generic – adjust to your actual crate names):

- `jks_premietac` – main TUI application:
  - loads songs from the database
  - shows a tree of songs grouped by type (left panel)
  - builds a “projection playlist” (right panel)
  - opens an external editor to edit or create songs
  - starts the presenter and monitor selector
- `tvoric_platna` – presenter GUI:
  - loads `temp_song_manager.json`
  - loads `monitor_config.json`
  - shows lyrics fullscreen on the selected monitor
- `monitor_selector` – helper GUI:
  - lets the user choose a monitor
  - saves monitor geometry (position and size) into `monitor_config.json`
- `prehladavac_db_jks` – shared library:
  - domain types (`TypPiesne`, `JKSTypPiesne`, `StrofaJKS`, `SongJks`, `SongManager`)
  - mapping between DB rows and domain model
  - helper functions to insert, delete, and load songs from SQLite

## Database and CSV import

The app uses SQLite, configured via `DATABASE_URL`:

```bash
export DATABASE_URL=./jks.sqlite
```

Songs can be imported from a `jks.csv` file using a one-off import binary that calls `init_db()`.

CSV format:

```text
id,cislo_strofy,typ,text
```

- `id`: numeric song ID
- `cislo_strofy`: stanza index (0 = title / first line)
- `typ`: textual type name (e.g. `Advent`, `Hymna`, or `none`)
- `text`: stanza text; `\n` sequences are converted to real newlines

Each stanza is stored as a row in the `jks` table; `SongManager` reconstructs full songs from these rows.

## TUI application

From the workspace root, run the main TUI app (adjust package name if needed):

```bash
cargo run -p jks_premietac
```

### Panels

The UI has three logical parts:

- **Left panel – database songs tree**
  - tree grouped by `TypPiesne` and `JKSTypPiesne`
  - search box at the top
- **Right panel – projection playlist**
  - linear list of songs selected for the presentation
- **Bottom bar – context help**
  - shows current key bindings depending on focus

### Key bindings

General:

- `q` or `Esc` – quit the application
- `Tab` – move focus (Left → Right → Search)
- `Shift+Tab` – move focus backwards

Left panel (database songs):

- `↑` / `↓` – move selection
- `←` / `→` – collapse / expand tree nodes
- `Space` – add selected song to the right playlist
- `Enter` – open selected song in an external editor and save changes back to DB
- `p` – create a new song (opens a temporary text file in an editor)
- `Delete` – delete selected song from the database (after confirmation)

Right panel (projection playlist):

- `↑` / `↓` – move selection
- `Space` – remove song from the playlist
- `Home` – start the presentation (saves playlist to JSON, runs `monitor_selector`, then presenter)

Search focus:

- typing – updates `song_lister.search` and filters the left panel
- `Backspace` – delete last character

The help bar at the bottom always shows the active key bindings for the currently focused panel.

## Presentation GUI

The presenter is usually started from within the TUI via the `Home` key (right panel):

1. The right-panel `SongManager` is saved to `temp_song_manager.json` in the same directory as the binaries.
2. `monitor_selector` is started to update/create `monitor_config.json`.
3. The presenter binary (`tvoric_platna`) is started.

The presenter:

- reads `temp_song_manager.json` (playlist)
- reads `monitor_config.json` (monitor geometry) with a fallback of 1280×720 at (0,0)
- creates a borderless fullscreen window on that monitor
- renders stanzas with as large a font as possible that still fits the screen

Controls in the presenter window:

- `Esc` – exit
- `Space` – toggle a manual black screen (hide/show text without changing the song)
- `→` – next stanza, or if at end of song, transition to black screen and then to the next song
- `←` – previous stanza, or transition to black screen and then previous song
- `↑` / `↓` – jump directly to the next/previous song (first stanza)

## Build and run

Basic setup:

1. Install Rust and Cargo.
2. Set `DATABASE_URL` for SQLite:
   ```bash
   export DATABASE_URL=./jks.sqlite
   ```
3. (Optional) Run the CSV import tool once to populate the database.
4. Build and run:

```bash
# Build all binaries
cargo build --release

# Run main TUI application
cargo run -p jks_premietac
```

The presenter (`tvoric_platna`) and monitor selector are started from inside the TUI based on the current executable path.