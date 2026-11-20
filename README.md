# GAnalyzer

GAnalyzer is a Windows desktop application for viewing G-code side by side with
human-readable command descriptions.

## Features

- Load `.gcode` files and view source lines beside translated commands.
- Tested and verified on files from Prusa Slicer and Orca Slicer.
- Use gcodes list and redlist command mappings to identify known commands and
  display `Unsupported command` for redlisted commands.
- Klipper printer (Voron) used for reference for redlisted commands.
- Detect missing translations and display `No translation`.
- Distinguish extrusion and retraction commands in translated linear moves.
- Hide embedded thumbnail data while preserving a placeholder in the view.
- Preview QOI and PNG thumbnails found in G-code files.
- Scroll large files while keeping corresponding rows aligned.

<img width="1047" height="1038" alt="screenshot" src="https://github.com/user-attachments/assets/d9b207a8-ee06-4735-9bdd-2fcd423f3dc2" />

Practical example: I am tuning a new slicer to work with my printer. It sends M142 which is not supported by Klipper. So I want to get rid of that. Also, there is a new code, M1234 which do not have a translation, add in gcode list or remove from slicer.

Upcoming ideas for next version:

It could be nice to have value in other color perhaps. Might do that as well.
Feature addon: A button to jump to next unsupported command/redlisted command could be nice


## Run

Run `ganalyzer.exe` on Windows. Keep the `docs` and `assets` folders alongside
the project files.

## Build from source

Install the Rust toolchain, open a terminal in the project folder, and run:

```powershell
cargo build --release
```

The executable is created at `target/release/ganalyzer.exe`.

The G-code command mappings are in `docs/gcodes.md` and
`docs/gcodes redlist.md`. GAnalyzer reads both files at startup.

## License

GAnalyzer is distributed under the MIT License. See [LICENSE](LICENSE).
