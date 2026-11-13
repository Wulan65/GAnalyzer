# GAnalyzer

GAnalyzer is a Windows desktop application for viewing G-code side by side with
human-readable command descriptions.

## Features

- Load `.gcode` files and view source lines beside translated commands.
- Tested and verified on files from Prusa Slicer and Orca Slicer.
- Use whitelist and redlist command mappings to identify known commands and
  display `Unsupported command` for redlisted commands.
- Klipper printer (Voron) used for reference for redlisted commands.
- Detect missing translations and display `No translation`.
- Distinguish extrusion and retraction commands in translated linear moves.
- Hide embedded thumbnail data while preserving a placeholder in the view.
- Preview QOI and PNG thumbnails found in G-code files.
- Scroll large files while keeping corresponding rows aligned.

<img width="1044" height="954" alt="screenshot" src="https://github.com/user-attachments/assets/9dc83883-8cab-4858-92dc-069c4c5c8667" />



## Run

Run `ganalyzer.exe` on Windows. Keep the `docs` and `assets` folders alongside
the project files.

## Build from source

Install the Rust toolchain, open a terminal in the project folder, and run:

```powershell
cargo build --release
```

The executable is created at `target/release/ganalyzer.exe`.

The G-code command mappings are in `docs/gcodes whitelist.md` and
`docs/gcodes redlist.md`. GAnalyzer reads both files at startup.

## License

GAnalyzer is distributed under the MIT License. See [LICENSE](LICENSE).
