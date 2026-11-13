# Application Requirements

## Product identity

- **Application name:** GAnalyzer
- Use `ganalyzer` as the Cargo package and executable name (`ganalyzer.exe` on Windows).
- Show the application name in the window title and taskbar.
- Use `assets/logo.svg` as the application icon in the window, taskbar, and Windows executable.
- Maintain the application name in this file. Rebuild after changing it.

## Platform and technology

- **Platform:** Windows desktop
- **Language:** Rust
- **GUI framework:** egui, hosted by eframe
- Do not display a console window at startup.

## Window

- Open at a default size of 1200 × 800 pixels.
- Allow the user to resize the window.

## User interface and behavior

- Show compact **Load** and **Quit** buttons in the toolbar.
- Show an embedded QOI or PNG thumbnail in the toolbar without changing the toolbar or button layout.
- Do not show the application name in the toolbar.
- **Load:** Open a file picker for `.gcode` files, display the selected file in the left panel, and show its light-gray (#CCCCCC) name slightly to the right of **Quit**, bottom-aligned with the button.
- Read the read-only `gcodes whitelist.md` CODE-to-description mapping from the application folder at each startup so edits apply after relaunch. For development runs, use `docs/gcodes whitelist.md` when the file is absent beside the executable.
- Read `gcodes redlist.md` from the application folder at startup and mark matching commands in the right panel as unsupported, including their descriptions. For development runs, use `docs/gcodes redlist.md` when the file is absent beside the executable.
- Show translated G-code and its original parameter values in the right panel without line numbers, preserving thumbnail placeholders.
- Render only visible G-code rows while scrolling large files.
- Keep corresponding rows vertically aligned between panels while scrolling; keep horizontal scrolling independent in each panel.
- **Quit:** Close the application.
- Keep button text at its default size.

## Formatting

- Display one non-wrapped line per file line, with line numbers in the left panel and a small gap before the content.
- Render G-code text and the filename in #CCCCCC, italic line numbers in #DCDC00 on a #696D75 background followed by two #696D75 spaces, and semicolon comment lines in #66B2FF.
- Hide sections from `; thumbnail begin` to `; thumbnail end`, from `; thumbnail_QOI begin` to `; thumbnail_QOI end`, and from `; THUMBNAIL_BLOCK_START` to `; THUMBNAIL_BLOCK_END`, preserving other line numbers and metadata that merely mentions thumbnails.
- Replace each group of hidden thumbnail sections, separated only by blank or lone `;` lines, with one #00d659 `Thumbnail section hidden` line aligned to each panel's G-code column. Trim adjacent blank and lone `;` lines, add an unnumbered blank row above and below the placeholder, and keep the left panel's gutter the same color as the panel background.
- For commands missing from `docs/gcodes whitelist.md`, render `No translation: ` in #fd54fd and keep the command and values in their normal text color (#CCCCCC).
- For commands listed in `docs/gcodes redlist.md`, render `Unsupported command: ` in #ec2525, followed by the listed description and original values in their normal text color (#CCCCCC), without repeating the raw command code.
- For translated `Linear move` commands, show `Extrude` when E has a value, `Extrude and linear move` when E and any of X/Y/Z have values, and `Retract` when any E value is negative; preserve all parameter values.
- When needed, show a 10 px horizontal scroll bar in each panel and a vertical scroll bar only in the right panel; use 50 px minimum scroll handles and keep them within their tracks. Align the right horizontal track's left inset with the left panel's and keep the vertical scrollbar fixed during horizontal scrolling.

## Visual design

- Use a modern medium-gray palette: window #46494E, content #3A3D42, and panels #4E5259.
- Make the toolbar (#303338) slightly darker than the main content area (#3A3D42).
- Use #696D75 for the divider at rest, #808994 on hover, and #949DA8 while dragging.
- Show two equal-sized panels side by side initially, with a draggable divider and 10 px outer padding. Keep the left panel width fixed when the window is resized; let the right panel absorb the width change.
- Keep the presentation clear, polished, and easy to use.

## Code and build process

- Keep the implementation compact and easy to follow.
- Build and verify the application with Cargo.
- Before every build, append a change summary to `docs/changes.md` in this format:
  `YYYY-MM-DD, HH:MM: [change description]`
- Use a 24-hour clock and limit each change description to 100 characters.
