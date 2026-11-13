2026-11-12, 21:30: Created application from template
2026-11-12, 21:45: Added equal side-by-side panels with 10 px spacing and a darker workspace background
2026-11-12, 21:45: Formatted the panel layout before final Cargo build
2026-11-12, 21:47: Removed main-area text and expanded the panels to use the available space
2026-11-12, 21:48: Added a draggable divider to resize the two panels
2026-11-12, 21:48: Formatted the panel splitter before the final build
2026-11-12, 21:53: Added G-code loading with numbered lines and persistent scroll bars
2026-11-12, 21:53: Corrected the egui scroll bar visibility type path
2026-11-12, 22:16: Set G-code text to white and line numbers to yellow italics
2026-11-12, 22:20: Show the loaded G-code filename to the right of Quit
2026-11-12, 22:22: Lightened and offset the toolbar filename label
2026-11-12, 22:23: Bottom-aligned the filename with the Quit button
2026-11-12, 22:25: Matched G-code text to the filename gray and darkened line numbers
2026-11-12, 22:30: Added blue semicolon comments and documented the display colors as hex codes
2026-11-12, 22:33: Hide thumbnail_QOI sections while preserving original line numbering
2026-11-12, 22:35: Match thumbnail markers that include metadata after the begin token
2026-11-12, 22:40: Recognize generic thumbnail begin and end markers
2026-11-12, 22:41: Restrict thumbnail filtering to block markers and keep later metadata visible
2026-11-12, 22:47: Show a red unnumbered placeholder for each hidden thumbnail section
2026-11-12, 22:50: Align the thumbnail placeholder with the G-code text column
2026-11-12, 22:56: Add runtime code translations and synchronized vertical panel scrolling
2026-11-12, 23:06: Highlight only the missing-translation label and match the selected accent color
2026-11-12, 23:09: Apply requirement colors to unknown commands and thumbnail placeholders
2026-11-12, 23:12: Apply the updated missing-translation red color
2026-11-12, 23:33: Added QOI thumbnail parsing and decoding
2026-11-12, 23:34: Rebuild after correcting QOI thumbnail handling
2026-11-12, 23:35: Verify thumbnail decoding against the sample G-code
2026-11-12, 23:35: Rebuild after validating sample thumbnail decoding
2026-11-12, 23:37: Enlarge the QOI thumbnail with 5 px toolbar spacing
2026-11-12, 23:39: Preserve toolbar layout while modestly enlarging the thumbnail
2026-11-12, 23:43: Keep the left panel width fixed while resizing the window
2026-11-12, 23:43: Build fixed-width panel resize behavior
2026-11-12, 23:49: Load PNG thumbnails from other slicer exports
2026-11-12, 23:50: Build PNG thumbnail support for the other slicer format
2026-11-12, 23:56: Virtualize G-code rows and avoid copying loaded file contents
2026-11-12, 23:56: Build virtualized G-code rendering
2026-11-12, 23:57: Keep test-only line adapter out of production builds
2026-11-12, 23:57: Rebuild without test-only warnings
2026-11-13, 00:01: Add hex codes to all named requirement colors
2026-11-13, 00:03: Set the default application width to 150 percent
2026-11-13, 00:05: Hide the left vertical scrollbar while preserving synchronized scrolling
2026-11-13, 00:06: Build left scrollbar visibility change
2026-11-13, 00:09: Fix nested vertical scroll synchronization and lighten the divider
2026-11-13, 00:09: Build scroll sync and divider color fix
2026-11-13, 00:11: Remove line numbers from translated G-code
2026-11-13, 00:12: Build right panel without row numbers
2026-11-13, 00:14: Normalize rendered row heights to keep panels synchronized
2026-11-13, 00:14: Build synchronized row layout fix
2026-11-13, 00:17: Hide nested THUMBNAIL_BLOCK sections as one placeholder
2026-11-13, 00:18: Build THUMBNAIL_BLOCK filtering support
2026-11-13, 00:20: Replace template identifiers with G-Code Viewer names
2026-11-13, 00:20: Build renamed G-Code Viewer application
2026-11-13, 00:22: Document the Cargo package and executable name
2026-11-13, 00:24: Align the right thumbnail placeholder with translated text
2026-11-13, 00:24: Build thumbnail placeholder alignment fix
2026-11-13, 00:25: Match the line-number background to the divider
2026-11-13, 00:25: Build line-number background change
2026-11-13, 00:26: Add gray dashes after source line numbers
2026-11-13, 00:26: Build gray line-number separators
2026-11-13, 00:28: Replace line-number dashes with gray spaces
2026-11-13, 00:33: Make scrollbar handles easier to grab with a 50 px minimum length
2026-11-13, 00:35: Keep horizontal scrollbar handles unchanged while enlarging the vertical handle
2026-11-13, 00:28: Build gray line-number spacing
2026-11-13, 00:30: Move line-number padding into its background and align placeholders
2026-11-13, 00:30: Build padded number column and placeholder alignment fix
2026-11-13, 00:38: Set all scrollbar handles to 50 px without changing track layout
2026-11-13, 00:41: Inset the right horizontal scrollbar track to keep its enlarged handle off the divider
2026-11-13, 00:50: Mask scrollbar handle overhangs while preserving visible track positions and sizes
2026-11-13, 00:50: Clip enlarged scrollbar handles to their tracks without moving the tracks
2026-11-13, 00:51: Use the widget corner radius when painting clipped scrollbar handles
2026-11-13, 00:59: Align the right horizontal track with the left and inset the vertical handle from the top
2026-11-13, 01:06: Keep the right vertical scrollbar fixed while horizontally scrolling
2026-11-13, 01:09: Remove the vertical handle's top inset so it meets the track edge
2026-11-13, 08:47: Trim and group hidden thumbnail sections, add blank spacing, and match the panel background
2026-11-13, 09:06: Read the redlisted G-code command list and mark matches as unsupported
2026-11-13, 09:07: Build redlisted-command warning support
2026-11-13, 09:11: Rename G-code lists and show redlist descriptions with the configured colors
2026-11-13, 09:12: Build renamed G-code lists and unsupported-command descriptions
2026-11-13, 09:14: Change the missing-translation prefix to "No translation: "
2026-11-13, 09:14: Build the updated missing-translation label
2026-11-13, 09:19: Show redlisted descriptions and values without repeating the command code
2026-11-13, 09:19: Build unsupported-command display without raw codes
2026-11-13, 09:26: Distinguish extrusion, combined moves, and retractions in translated linear moves
2026-11-13, 09:26: Build linear extrusion and retraction labeling
2026-11-13, 10:33: Rename app to GAnalyzer and apply SVG logo to window and Windows executable icons
2026-11-13, 10:35: Build GAnalyzer and verify Windows icon resource integration
2026-11-13, 10:44: Update the documented logo path after moving the SVG into assets
2026-11-13, 10:45: Update application references to Markdown files moved into docs
2026-11-13, 17:06: Load G-code lists beside the executable, with a development fallback to docs/
