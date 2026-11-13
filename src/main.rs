#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use base64::{engine::general_purpose, Engine as _};
use eframe::egui::{self, Color32, ColorImage, TextureHandle};
use std::collections::HashMap;
use std::path::Path;

const REQUIREMENTS: &str = include_str!("../docs/requirements.md");

fn decode_qoi(data: &[u8], expected_width: usize, expected_height: usize) -> Option<Vec<u8>> {
    if data.len() < 22 || &data[0..4] != b"qoif" {
        return None;
    }

    let width = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    let height = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);
    let channels = data[12];
    let colorspace = data[13];

    if channels != 3 && channels != 4 {
        return None;
    }
    if colorspace != 0 && colorspace != 1 {
        return None;
    }
    if width as usize != expected_width || height as usize != expected_height {
        return None;
    }

    let pixel_count = (width as usize).checked_mul(height as usize)?;
    let mut pixels = vec![0u8; pixel_count.checked_mul(4)?];
    let mut index = 14usize;
    let mut run = 0u8;
    let mut rgba = [0u8, 0u8, 0u8, 255u8];
    let mut lookup = [[0u8; 4]; 64];

    for i in 0..pixel_count {
        if run > 0 {
            run -= 1;
        } else {
            let byte = *data.get(index)?;
            index += 1;

            if byte == 0xFE {
                let rgb = data.get(index..index.checked_add(3)?)?;
                rgba[..3].copy_from_slice(rgb);
                index += 3;
            } else if byte == 0xFF {
                let next_rgba = data.get(index..index.checked_add(4)?)?;
                rgba.copy_from_slice(next_rgba);
                index += 4;
            } else if byte & 0xC0 == 0x00 {
                rgba = lookup[byte as usize];
            } else if byte & 0xC0 == 0x40 {
                let dr = ((byte >> 4) & 0x03) as i16 - 2;
                let dg = ((byte >> 2) & 0x03) as i16 - 2;
                let db = (byte & 0x03) as i16 - 2;
                rgba[0] = (rgba[0] as i16 + dr) as u8;
                rgba[1] = (rgba[1] as i16 + dg) as u8;
                rgba[2] = (rgba[2] as i16 + db) as u8;
            } else if byte & 0xC0 == 0x80 {
                let byte2 = *data.get(index)?;
                index += 1;
                let dg = (byte & 0x3F) as i16 - 32;
                let dr_dg = ((byte2 >> 4) & 0x0F) as i16 - 8;
                let db_dg = (byte2 & 0x0F) as i16 - 8;
                rgba[0] = (rgba[0] as i16 + dg + dr_dg) as u8;
                rgba[1] = (rgba[1] as i16 + dg) as u8;
                rgba[2] = (rgba[2] as i16 + dg + db_dg) as u8;
            } else {
                run = byte & 0x3F;
                if run as usize > pixel_count - i - 1 {
                    return None;
                }
            }
        }

        let hash = (rgba[0]
            .wrapping_mul(3)
            .wrapping_add(rgba[1].wrapping_mul(5))
            .wrapping_add(rgba[2].wrapping_mul(7))
            .wrapping_add(rgba[3].wrapping_mul(11)))
            % 64;
        lookup[hash as usize] = rgba;

        let pixel = &mut pixels[i * 4..i * 4 + 4];
        pixel.copy_from_slice(&rgba);
        if channels == 3 {
            pixel[3] = 255;
        }
    }

    Some(pixels)
}

fn extract_thumbnails(contents: &str) -> Vec<(usize, usize, Vec<u8>)> {
    let mut thumbnails = Vec::new();
    let mut lines = contents.lines().peekable();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        let Some((thumbnail_type, action)) = thumbnail_marker(trimmed) else {
            continue;
        };
        if !action.eq_ignore_ascii_case("begin") {
            continue;
        }

        let mut marker_parts = trimmed.trim_start_matches(';').split_whitespace();
        marker_parts.next();
        marker_parts.next();
        let Some(dims) = marker_parts.next() else {
            continue;
        };
        {
            let Some((width, height)) = dims.split_once('x') else {
                continue;
            };
            let width: usize = width.parse().unwrap_or(0);
            let height: usize = height.parse().unwrap_or(0);

            if width == 0 || height == 0 {
                continue;
            }

            let mut encoded_data = Vec::<&str>::new();
            while let Some(data_line) = lines.peek() {
                let data_trimmed = data_line.trim();
                if let Some((end_type, end_action)) = thumbnail_marker(data_trimmed) {
                    if end_type.eq_ignore_ascii_case(thumbnail_type)
                        && end_action.eq_ignore_ascii_case("end")
                    {
                        lines.next();
                        break;
                    }
                }
                if data_trimmed.starts_with(';') {
                    lines.next();
                    encoded_data.push(data_trimmed);
                } else {
                    break;
                }
            }

            if !encoded_data.is_empty() {
                let base64_data: String = encoded_data
                    .iter()
                    .map(|s| s.trim_start_matches(';').trim())
                    .collect();

                if let Ok(decoded) = general_purpose::STANDARD.decode(base64_data) {
                    thumbnails.push((width, height, decoded));
                }
            }
        }
    }

    thumbnails
}

fn decode_thumbnail(width: usize, height: usize, data: &[u8]) -> Option<ColorImage> {
    let pixels = if data.starts_with(b"qoif") {
        decode_qoi(data, width, height)?
    } else {
        let image = image::load_from_memory(data).ok()?;
        if image.width() as usize != width || image.height() as usize != height {
            return None;
        }
        image.to_rgba8().into_raw()
    };

    Some(ColorImage::from_rgba_unmultiplied([width, height], &pixels))
}

fn load_thumbnail(contents: &str) -> Option<ColorImage> {
    let mut thumbnails = extract_thumbnails(contents);
    thumbnails.sort_by_key(|(width, height, _)| std::cmp::Reverse(width.saturating_mul(*height)));
    thumbnails
        .into_iter()
        .find_map(|(width, height, data)| decode_thumbnail(width, height, &data))
}

fn application_name() -> &'static str {
    REQUIREMENTS
        .lines()
        .find_map(|line| line.strip_prefix("- **Application name:**"))
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .expect("docs/requirements.md must define an application name")
}

#[cfg(test)]
#[derive(Debug, PartialEq, Eq)]
enum GcodeDisplayLine<'a> {
    FileLine { line_number: usize, text: &'a str },
    ThumbnailSpacer,
    ThumbnailHidden,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GcodeDisplayRow {
    FileLine {
        line_number: usize,
        start: usize,
        end: usize,
    },
    ThumbnailSpacer,
    ThumbnailHidden,
}

fn gcode_display_rows(contents: &str) -> Vec<GcodeDisplayRow> {
    let mut rows = Vec::new();
    if contents.is_empty() {
        return rows;
    }

    let mut thumbnail_depth = 0;
    let mut offset = 0;

    for (line_number, raw_line) in contents.split_inclusive('\n').enumerate() {
        let line = raw_line.strip_suffix('\n').unwrap_or(raw_line);
        let line = if raw_line.ends_with('\n') {
            line.strip_suffix('\r').unwrap_or(line)
        } else {
            line
        };

        match thumbnail_marker_action(line) {
            Some(action) if action.eq_ignore_ascii_case("begin") => {
                if thumbnail_depth == 0 {
                    rows.push(GcodeDisplayRow::ThumbnailHidden);
                }
                thumbnail_depth += 1;
            }
            Some(action) if action.eq_ignore_ascii_case("end") && thumbnail_depth > 0 => {
                thumbnail_depth -= 1;
            }
            _ if thumbnail_depth > 0 => {}
            _ => rows.push(GcodeDisplayRow::FileLine {
                line_number,
                start: offset,
                end: offset + line.len(),
            }),
        }
        offset += raw_line.len();
    }

    let is_thumbnail_separator = |row: GcodeDisplayRow| match row {
        GcodeDisplayRow::FileLine { start, end, .. } => {
            matches!(contents[start..end].trim(), "" | ";")
        }
        GcodeDisplayRow::ThumbnailSpacer | GcodeDisplayRow::ThumbnailHidden => false,
    };
    let mut compact_rows = Vec::with_capacity(rows.len());
    let mut index = 0;
    while index < rows.len() {
        if rows[index] == GcodeDisplayRow::ThumbnailHidden {
            while compact_rows
                .last()
                .copied()
                .map(&is_thumbnail_separator)
                .unwrap_or(false)
            {
                compact_rows.pop();
            }
            compact_rows.push(GcodeDisplayRow::ThumbnailHidden);
            index += 1;

            while index < rows.len() {
                while index < rows.len() && is_thumbnail_separator(rows[index]) {
                    index += 1;
                }
                if index < rows.len() && rows[index] == GcodeDisplayRow::ThumbnailHidden {
                    index += 1;
                } else {
                    break;
                }
            }
        } else {
            compact_rows.push(rows[index]);
            index += 1;
        }
    }

    let mut display_rows = Vec::with_capacity(compact_rows.len() + 2);
    for row in compact_rows {
        if row == GcodeDisplayRow::ThumbnailHidden {
            display_rows.push(GcodeDisplayRow::ThumbnailSpacer);
            display_rows.push(row);
            display_rows.push(GcodeDisplayRow::ThumbnailSpacer);
        } else {
            display_rows.push(row);
        }
    }

    display_rows
}

#[cfg(test)]
fn gcode_display_lines(contents: &str) -> impl Iterator<Item = GcodeDisplayLine<'_>> {
    gcode_display_rows(contents)
        .into_iter()
        .map(move |row| match row {
            GcodeDisplayRow::FileLine {
                line_number,
                start,
                end,
            } => GcodeDisplayLine::FileLine {
                line_number,
                text: &contents[start..end],
            },
            GcodeDisplayRow::ThumbnailSpacer => GcodeDisplayLine::ThumbnailSpacer,
            GcodeDisplayRow::ThumbnailHidden => GcodeDisplayLine::ThumbnailHidden,
        })
}

fn thumbnail_marker(line: &str) -> Option<(&str, &str)> {
    let mut tokens = line.trim().strip_prefix(';')?.split_whitespace();
    let name = tokens.next()?;
    if !name.eq_ignore_ascii_case("thumbnail") && !name.eq_ignore_ascii_case("thumbnail_QOI") {
        return None;
    }
    Some((name, tokens.next()?))
}

fn thumbnail_marker_action(line: &str) -> Option<&str> {
    let mut tokens = line.trim().strip_prefix(';')?.split_whitespace();
    let name = tokens.next()?;
    if name.eq_ignore_ascii_case("THUMBNAIL_BLOCK_START") {
        Some("begin")
    } else if name.eq_ignore_ascii_case("THUMBNAIL_BLOCK_END") {
        Some("end")
    } else if name.eq_ignore_ascii_case("thumbnail") || name.eq_ignore_ascii_case("thumbnail_QOI") {
        tokens.next()
    } else {
        None
    }
}

fn load_gcode_whitelist() -> Result<HashMap<String, String>, String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/gcodes whitelist.md");
    let contents = std::fs::read_to_string(&path)
        .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
    parse_gcode_translations(&contents).map_err(|error| format!("{}: {error}", path.display()))
}

fn load_gcode_redlist() -> Result<HashMap<String, String>, String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/gcodes redlist.md");
    let contents = std::fs::read_to_string(&path)
        .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
    parse_gcode_translations(&contents).map_err(|error| format!("{}: {error}", path.display()))
}

fn parse_gcode_translations(contents: &str) -> Result<HashMap<String, String>, String> {
    let mut translations = HashMap::new();
    for (line_number, line) in contents.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some((command, description)) = line.split_once(':') else {
            return Err(format!(
                "line {} must have the form CODE: description",
                line_number + 1
            ));
        };
        let command = command.trim();
        let description = description.trim();
        if command.is_empty() || description.is_empty() {
            return Err(format!(
                "line {} must have a command and description",
                line_number + 1
            ));
        }
        translations.insert(command.to_ascii_uppercase(), description.to_owned());
    }
    if translations.is_empty() {
        return Err("the file contains no G-code entries".to_owned());
    }
    Ok(translations)
}

fn split_gcode_command(line: &str) -> Option<(&str, &str)> {
    let code = line.split(';').next()?.trim();
    let mut words = code.split_whitespace();
    let first_word = words.next()?;
    let (command_word, command_start) = if first_word.strip_prefix('N').map_or(false, |number| {
        !number.is_empty() && number.bytes().all(|byte| byte.is_ascii_digit())
    }) {
        let command_word = words.next()?;
        (command_word, code.find(command_word)?)
    } else {
        (first_word, 0)
    };

    let bytes = command_word.as_bytes();
    let command_length = if bytes.first().map_or(false, |byte| {
        matches!(byte.to_ascii_uppercase(), b'G' | b'M')
    }) {
        let mut end = 1;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
        if end == 1 {
            return None;
        }
        if end < bytes.len() && bytes[end] == b'.' {
            end += 1;
            let fractional_start = end;
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
            if end == fractional_start {
                return None;
            }
        }
        end
    } else {
        command_word.len()
    };

    let command_end = command_start + command_length;
    Some((
        &code[command_start..command_end],
        code[command_end..].trim(),
    ))
}

fn parse_gcode_parameter_number(bytes: &[u8], start: usize) -> Option<(f64, usize)> {
    let mut end = start;
    if bytes
        .get(end)
        .map_or(false, |byte| matches!(byte, b'+' | b'-'))
    {
        end += 1;
    }

    let mut digit_count = 0;
    while bytes.get(end).map_or(false, u8::is_ascii_digit) {
        digit_count += 1;
        end += 1;
    }
    if bytes.get(end) == Some(&b'.') {
        end += 1;
        while bytes.get(end).map_or(false, u8::is_ascii_digit) {
            digit_count += 1;
            end += 1;
        }
    }
    if digit_count == 0 {
        return None;
    }

    let number = std::str::from_utf8(&bytes[start..end]).ok()?.parse().ok()?;
    Some((number, end))
}

fn linear_move_extrusion_description(parameters: &str) -> Option<&'static str> {
    let bytes = parameters.as_bytes();
    let mut has_xyz = false;
    let mut has_extrusion = false;
    let mut has_negative_extrusion = false;
    let mut index = 0;

    while index < bytes.len() {
        let axis = bytes[index].to_ascii_uppercase();
        if bytes[index].is_ascii_alphabetic() {
            if let Some((value, end)) = parse_gcode_parameter_number(bytes, index + 1) {
                match axis {
                    b'X' | b'Y' | b'Z' => has_xyz = true,
                    b'E' => {
                        has_extrusion = true;
                        has_negative_extrusion |= value < 0.0;
                    }
                    _ => {}
                }
                index = end;
                continue;
            }
        }
        index += 1;
    }

    if has_negative_extrusion {
        Some("Retract")
    } else if has_extrusion && has_xyz {
        Some("Extrude and linear move")
    } else if has_extrusion {
        Some("Extrude")
    } else {
        None
    }
}

struct TranslatedGcodeLine {
    text: String,
    missing_translation: bool,
    unsupported_command: bool,
}

fn translate_gcode_line(
    line: &str,
    translations: &HashMap<String, String>,
    redlisted_commands: &HashMap<String, String>,
) -> TranslatedGcodeLine {
    let code = line.split(';').next().unwrap_or_default().trim();
    if code.is_empty() {
        return TranslatedGcodeLine {
            text: line.trim().to_owned(),
            missing_translation: false,
            unsupported_command: false,
        };
    }
    let Some((command, parameters)) = split_gcode_command(code) else {
        return TranslatedGcodeLine {
            text: code.to_owned(),
            missing_translation: false,
            unsupported_command: false,
        };
    };
    if let Some(description) = redlisted_commands.get(&command.to_ascii_uppercase()) {
        let suffix = if parameters.is_empty() {
            String::new()
        } else {
            format!(" {parameters}")
        };
        return TranslatedGcodeLine {
            text: format!("Unsupported command: {description}{suffix}"),
            missing_translation: false,
            unsupported_command: true,
        };
    }
    let (description, missing_translation) =
        if let Some(description) = translations.get(&command.to_ascii_uppercase()) {
            (description.clone(), false)
        } else {
            (format!("No translation: {command}"), true)
        };
    let description = if !missing_translation && description.eq_ignore_ascii_case("Linear move") {
        linear_move_extrusion_description(parameters)
            .map(str::to_owned)
            .unwrap_or(description)
    } else {
        description
    };
    let text = if parameters.is_empty() {
        description
    } else {
        format!("{description} {parameters}")
    };
    TranslatedGcodeLine {
        text,
        missing_translation,
        unsupported_command: false,
    }
}

fn render_line_number(ui: &mut egui::Ui, line_number: usize) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.add(
            egui::Label::new(
                egui::RichText::new(format!("{:>6}  ", line_number + 1))
                    .monospace()
                    .color(Color32::from_rgb(220, 220, 0))
                    .background_color(Color32::from_rgb(105, 109, 117))
                    .italics(),
            )
            .extend()
            .selectable(true),
        );
    });
}

fn render_gcode_row(ui: &mut egui::Ui, line_number: usize, text: &str, color: Color32) {
    ui.horizontal(|ui| {
        render_line_number(ui, line_number);
        render_gcode_text(ui, text, color);
    });
}

fn render_gcode_text(ui: &mut egui::Ui, text: &str, color: Color32) {
    ui.horizontal(|ui| {
        ui.add(
            egui::Label::new(
                egui::RichText::new(format!("  {text}"))
                    .monospace()
                    .color(color),
            )
            .extend()
            .selectable(true),
        );
    });
}

fn render_gcode_status_text(
    ui: &mut egui::Ui,
    prefix: &str,
    suffix: &str,
    prefix_color: Color32,
    suffix_color: Color32,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.add(
            egui::Label::new(
                egui::RichText::new(format!("  {prefix}"))
                    .monospace()
                    .color(prefix_color),
            )
            .extend()
            .selectable(true),
        );
        ui.add(
            egui::Label::new(egui::RichText::new(suffix).monospace().color(suffix_color))
                .extend()
                .selectable(true),
        );
    });
}

fn render_thumbnail_placeholder(ui: &mut egui::Ui, translated: bool) {
    let placeholder_color = Color32::from_rgb(0, 214, 89);
    if translated {
        render_gcode_text(ui, "Thumbnail section hidden", placeholder_color);
    } else {
        ui.horizontal(|ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.add(egui::Label::new(egui::RichText::new("      ").monospace()).extend());
                ui.add(egui::Label::new(egui::RichText::new("  ").monospace()).extend());
            });
            render_gcode_text(ui, "Thumbnail section hidden", placeholder_color);
        });
    }
}

fn max_gcode_row_chars(
    contents: &str,
    rows: &[GcodeDisplayRow],
    translations: &HashMap<String, String>,
    redlisted_commands: &HashMap<String, String>,
    translated: bool,
) -> usize {
    rows.iter()
        .map(|row| match row {
            GcodeDisplayRow::ThumbnailSpacer => 0,
            GcodeDisplayRow::ThumbnailHidden => {
                (if translated { 2 } else { 10 }) + "Thumbnail section hidden".chars().count()
            }
            GcodeDisplayRow::FileLine {
                line_number,
                start,
                end,
            } => {
                let text = &contents[*start..*end];
                let text_chars = if translated {
                    translate_gcode_line(text, translations, redlisted_commands)
                        .text
                        .chars()
                        .count()
                } else {
                    text.chars().count()
                };
                let number_chars = if translated {
                    0
                } else {
                    6.max((line_number + 1).to_string().len())
                };
                number_chars + 4 + text_chars
            }
        })
        .max()
        .unwrap_or(0)
}

fn render_gcode_rows(
    ui: &mut egui::Ui,
    contents: &str,
    rows: &[GcodeDisplayRow],
    visible_rows: std::ops::Range<usize>,
    row_height: f32,
    translations: &HashMap<String, String>,
    redlisted_commands: &HashMap<String, String>,
    translated: bool,
) {
    for display_row in &rows[visible_rows] {
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), row_height),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| match *display_row {
                GcodeDisplayRow::ThumbnailSpacer => {}
                GcodeDisplayRow::ThumbnailHidden => render_thumbnail_placeholder(ui, translated),
                GcodeDisplayRow::FileLine {
                    line_number,
                    start,
                    end,
                } => {
                    let text = &contents[start..end];
                    let color = if text.trim_start().starts_with(';') {
                        Color32::from_rgb(102, 178, 255)
                    } else {
                        Color32::from_gray(204)
                    };
                    if translated {
                        let translation =
                            translate_gcode_line(text, translations, redlisted_commands);
                        if translation.unsupported_command {
                            let suffix = translation
                                .text
                                .strip_prefix("Unsupported command: ")
                                .unwrap_or_default();
                            render_gcode_status_text(
                                ui,
                                "Unsupported command: ",
                                suffix,
                                Color32::from_rgb(236, 37, 37),
                                color,
                            );
                        } else if translation.missing_translation {
                            let suffix = translation
                                .text
                                .strip_prefix("No translation: ")
                                .unwrap_or_default();
                            render_gcode_status_text(
                                ui,
                                "No translation: ",
                                suffix,
                                Color32::from_rgb(253, 84, 253),
                                color,
                            );
                        } else {
                            render_gcode_text(ui, &translation.text, color);
                        }
                    } else {
                        render_gcode_row(ui, line_number, text, color);
                    }
                }
            },
        );
    }
}

fn main() -> eframe::Result<()> {
    let name = application_name();
    let icon = image::load_from_memory(include_bytes!("../assets/ganalyzer.png"))
        .expect("embedded application icon must be a valid PNG")
        .to_rgba8();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([420.0, 320.0])
            .with_icon(egui::IconData {
                rgba: icon.as_raw().to_vec(),
                width: icon.width(),
                height: icon.height(),
            })
            .with_title(name),
        ..Default::default()
    };

    eframe::run_native(
        name,
        options,
        Box::new(|creation_context| {
            let mut visuals = egui::Visuals::dark();
            visuals.panel_fill = Color32::from_rgb(70, 73, 78);
            visuals.window_fill = Color32::from_rgb(70, 73, 78);
            visuals.widgets.inactive.bg_fill = Color32::from_rgb(91, 95, 102);
            visuals.widgets.hovered.bg_fill = Color32::from_rgb(112, 119, 128);
            visuals.widgets.active.bg_fill = Color32::from_rgb(128, 137, 148);
            creation_context.egui_ctx.set_visuals(visuals);
            Ok(Box::new(GCodeViewerApp::default()))
        }),
    )
}

struct GCodeViewerApp {
    left_panel_width: Option<f32>,
    gcode_content: Option<String>,
    display_rows: Vec<GcodeDisplayRow>,
    left_max_row_chars: usize,
    right_max_row_chars: usize,
    load_error: Option<String>,
    loaded_filename: Option<String>,
    gcode_translations: HashMap<String, String>,
    redlisted_commands: HashMap<String, String>,
    display_config_error: Option<String>,
    vertical_scroll_offset: f32,
    thumbnail_texture: Option<TextureHandle>,
}

impl Default for GCodeViewerApp {
    fn default() -> Self {
        let mut config_errors = Vec::new();
        let gcode_translations = match load_gcode_whitelist() {
            Ok(entries) => entries,
            Err(error) => {
                config_errors.push(error);
                HashMap::new()
            }
        };
        let redlisted_commands = match load_gcode_redlist() {
            Ok(entries) => entries,
            Err(error) => {
                config_errors.push(error);
                HashMap::new()
            }
        };
        let display_config_error = if config_errors.is_empty() {
            None
        } else {
            Some(config_errors.join("\n"))
        };
        Self {
            left_panel_width: None,
            gcode_content: None,
            display_rows: Vec::new(),
            left_max_row_chars: 0,
            right_max_row_chars: 0,
            load_error: None,
            loaded_filename: None,
            gcode_translations,
            redlisted_commands,
            display_config_error,
            vertical_scroll_offset: 0.0,
            thumbnail_texture: None,
        }
    }
}

fn scrollbar_track_rect(
    inner_rect: egui::Rect,
    outer_rect: egui::Rect,
    mapping_rect: egui::Rect,
    horizontal: bool,
    inner_margin: f32,
    outer_margin: f32,
) -> egui::Rect {
    if horizontal {
        egui::Rect::from_min_max(
            egui::pos2(mapping_rect.min.x, inner_rect.max.y + inner_margin),
            egui::pos2(mapping_rect.max.x, outer_rect.max.y - outer_margin),
        )
    } else {
        egui::Rect::from_min_max(
            egui::pos2(inner_rect.max.x + inner_margin, mapping_rect.min.y),
            egui::pos2(outer_rect.max.x - outer_margin, mapping_rect.max.y),
        )
    }
}

fn mask_scrollbar_handle_overflow(
    painter: &egui::Painter,
    panel_rect: egui::Rect,
    track_rect: egui::Rect,
    horizontal: bool,
    panel_color: Color32,
) {
    let painter = painter.with_clip_rect(panel_rect);
    if horizontal {
        let left_mask = egui::Rect::from_min_max(
            egui::pos2(panel_rect.min.x, track_rect.min.y),
            egui::pos2(track_rect.min.x.min(panel_rect.max.x), track_rect.max.y),
        );
        if left_mask.width() > 0.0 {
            painter.rect_filled(left_mask, egui::CornerRadius::ZERO, panel_color);
        }

        let right_mask = egui::Rect::from_min_max(
            egui::pos2(track_rect.max.x.max(panel_rect.min.x), track_rect.min.y),
            egui::pos2(panel_rect.max.x, track_rect.max.y),
        );
        if right_mask.width() > 0.0 {
            painter.rect_filled(right_mask, egui::CornerRadius::ZERO, panel_color);
        }
    } else {
        let top_mask = egui::Rect::from_min_max(
            egui::pos2(track_rect.min.x, panel_rect.min.y),
            egui::pos2(track_rect.max.x, track_rect.min.y.min(panel_rect.max.y)),
        );
        if top_mask.height() > 0.0 {
            painter.rect_filled(top_mask, egui::CornerRadius::ZERO, panel_color);
        }

        let bottom_mask = egui::Rect::from_min_max(
            egui::pos2(track_rect.min.x, track_rect.max.y.max(panel_rect.min.y)),
            egui::pos2(track_rect.max.x, panel_rect.max.y),
        );
        if bottom_mask.height() > 0.0 {
            painter.rect_filled(bottom_mask, egui::CornerRadius::ZERO, panel_color);
        }
    }
}

fn paint_scrollbar(
    ui: &egui::Ui,
    panel_rect: egui::Rect,
    track_rect: egui::Rect,
    mapping_rect: egui::Rect,
    horizontal: bool,
    offset: f32,
    content_size: f32,
    viewport_size: f32,
    style: egui::style::ScrollStyle,
) {
    let (track_start, track_end) = if horizontal {
        (mapping_rect.min.x, mapping_rect.max.x)
    } else {
        (mapping_rect.min.y, mapping_rect.max.y)
    };
    let track_length = track_end - track_start;
    let content_size = content_size.max(viewport_size);
    if track_length <= 0.0 || content_size <= 0.0 {
        return;
    }

    let offset = offset.clamp(0.0, (content_size - viewport_size).max(0.0));
    let raw_handle_length = (viewport_size / content_size) * track_length;
    let handle_length = raw_handle_length
        .max(style.handle_min_length)
        .min(track_length);
    let raw_handle_start = track_start + (offset / content_size) * track_length;
    let handle_start = (raw_handle_start + raw_handle_length / 2.0 - handle_length / 2.0)
        .clamp(track_start, track_end - handle_length);
    let handle_rect = if horizontal {
        egui::Rect::from_min_max(
            egui::pos2(handle_start, track_rect.min.y),
            egui::pos2(handle_start + handle_length, track_rect.max.y),
        )
    } else {
        egui::Rect::from_min_max(
            egui::pos2(track_rect.min.x, handle_start),
            egui::pos2(track_rect.max.x, handle_start + handle_length),
        )
    };

    let visuals = ui.visuals();
    let handle_visuals =
        if ui.input(|input| input.pointer.primary_down()) && ui.rect_contains_pointer(track_rect) {
            &visuals.widgets.active
        } else if ui.rect_contains_pointer(handle_rect) {
            &visuals.widgets.hovered
        } else {
            &visuals.widgets.inactive
        };
    let handle_color = if style.foreground_color {
        handle_visuals.fg_stroke.color
    } else {
        handle_visuals.bg_fill
    };
    let painter = ui
        .painter()
        .with_clip_rect(panel_rect)
        .with_clip_rect(track_rect);
    painter.rect_filled(
        track_rect,
        handle_visuals.corner_radius,
        visuals.extreme_bg_color,
    );
    painter.rect_filled(handle_rect, handle_visuals.corner_radius, handle_color);
}

impl eframe::App for GCodeViewerApp {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("toolbar")
            .frame(
                egui::Frame::default()
                    .fill(Color32::from_rgb(48, 51, 56))
                    .inner_margin(egui::Margin::same(14)),
            )
            .show(context, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Max), |ui| {
                    if ui
                        .add_sized([70.0, 30.0], egui::Button::new("Load"))
                        .clicked()
                    {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("G-code", &["gcode"])
                            .pick_file()
                        {
                            let is_gcode = path
                                .extension()
                                .and_then(|extension| extension.to_str())
                                .map_or(false, |extension| extension.eq_ignore_ascii_case("gcode"));
                            if !is_gcode {
                                self.gcode_content = None;
                                self.display_rows.clear();
                                self.left_max_row_chars = 0;
                                self.right_max_row_chars = 0;
                                self.loaded_filename = None;
                                self.load_error =
                                    Some("Please select a file with the .gcode extension.".into());
                                self.thumbnail_texture = None;
                            } else {
                                match std::fs::read_to_string(&path) {
                                    Ok(contents) => {
                                        let display_rows = gcode_display_rows(&contents);
                                        self.left_max_row_chars = max_gcode_row_chars(
                                            &contents,
                                            &display_rows,
                                            &self.gcode_translations,
                                            &self.redlisted_commands,
                                            false,
                                        );
                                        self.right_max_row_chars = max_gcode_row_chars(
                                            &contents,
                                            &display_rows,
                                            &self.gcode_translations,
                                            &self.redlisted_commands,
                                            true,
                                        );
                                        let thumbnail_texture =
                                            load_thumbnail(&contents).map(|color_image| {
                                                context.load_texture(
                                                    "thumbnail",
                                                    color_image,
                                                    Default::default(),
                                                )
                                            });
                                        self.display_rows = display_rows;
                                        self.gcode_content = Some(contents);
                                        self.loaded_filename = path
                                            .file_name()
                                            .map(|name| name.to_string_lossy().into_owned());
                                        self.load_error = None;
                                        self.vertical_scroll_offset = 0.0;
                                        self.thumbnail_texture = thumbnail_texture;
                                    }
                                    Err(error) => {
                                        self.gcode_content = None;
                                        self.display_rows.clear();
                                        self.left_max_row_chars = 0;
                                        self.right_max_row_chars = 0;
                                        self.loaded_filename = None;
                                        self.load_error = Some(format!(
                                            "Could not read {}: {error}",
                                            path.display()
                                        ));
                                        self.thumbnail_texture = None;
                                    }
                                }
                            }
                        }
                    }
                    let quit_response = ui.add_sized([70.0, 30.0], egui::Button::new("Quit"));
                    if quit_response.clicked() {
                        context.send_viewport_cmd(egui::ViewportCommand::Close);
                    }

                    if let Some(texture) = &self.thumbnail_texture {
                        ui.add_space(8.0);
                        let image_height = 35.0;
                        let image_size =
                            texture.size_vec2() * (image_height / texture.size_vec2().y);
                        let image_rect = egui::Rect::from_center_size(
                            egui::pos2(
                                ui.cursor().min.x + image_size.x / 2.0,
                                quit_response.rect.center().y,
                            ),
                            image_size,
                        );
                        ui.painter().image(
                            texture.id(),
                            image_rect,
                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                            Color32::WHITE,
                        );
                        ui.add_space(image_size.x);
                    }

                    if let Some(filename) = self.loaded_filename.as_deref() {
                        ui.add_space(8.0);
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(filename).color(Color32::from_gray(204)),
                            )
                            .truncate(),
                        );
                    }
                });
            });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .fill(Color32::from_rgb(58, 61, 66))
                    .inner_margin(egui::Margin::same(10)),
            )
            .show(context, |ui| {
                let panels_rect = ui.available_rect_before_wrap();
                let divider_width = 10.0;
                let available_width = panels_rect.width() - divider_width;
                let min_panel_width = 80.0;
                let max_left_width = available_width - min_panel_width;
                let requested_left_width = *self
                    .left_panel_width
                    .get_or_insert(available_width * 0.5);
                let left_width =
                    requested_left_width.clamp(min_panel_width, max_left_width);
                let divider_rect = egui::Rect::from_min_size(
                    egui::pos2(panels_rect.min.x + left_width, panels_rect.min.y),
                    egui::vec2(divider_width, panels_rect.height()),
                );

                ui.allocate_rect(panels_rect, egui::Sense::hover());
                let divider_response = ui.interact(
                    divider_rect,
                    ui.make_persistent_id("panel_splitter"),
                    egui::Sense::click_and_drag(),
                );
                if divider_response.dragged() {
                    if let Some(pointer_position) = divider_response.interact_pointer_pos() {
                        self.left_panel_width = Some(
                            (pointer_position.x - panels_rect.min.x - divider_width / 2.0)
                                .clamp(min_panel_width, max_left_width),
                        );
                    }
                }
                if divider_response.hovered() || divider_response.dragged() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                }

                let left_width = self
                    .left_panel_width
                    .unwrap_or(available_width * 0.5)
                    .clamp(min_panel_width, max_left_width);
                let left_panel = egui::Rect::from_min_size(
                    panels_rect.min,
                    egui::vec2(left_width, panels_rect.height()),
                );
                let divider_rect = egui::Rect::from_min_size(
                    egui::pos2(panels_rect.min.x + left_width, panels_rect.min.y),
                    egui::vec2(divider_width, panels_rect.height()),
                );
                let right_panel = egui::Rect::from_min_size(
                    egui::pos2(divider_rect.max.x, panels_rect.min.y),
                    egui::vec2(panels_rect.max.x - divider_rect.max.x, panels_rect.height()),
                );

                let panel_color = Color32::from_rgb(78, 82, 89);
                let corner_radius = egui::CornerRadius::same(6);
                ui.painter()
                    .rect_filled(left_panel, corner_radius, panel_color);
                ui.painter()
                    .rect_filled(right_panel, corner_radius, panel_color);
                let mut scrollbar_style = ui.style().spacing.scroll;
                scrollbar_style.bar_width = 10.0;
                scrollbar_style.handle_min_length = 50.0;
                scrollbar_style.floating = false;
                let divider_color = if divider_response.dragged() {
                    Color32::from_rgb(148, 157, 168)
                } else if divider_response.hovered() {
                    Color32::from_rgb(128, 137, 148)
                } else {
                    Color32::from_rgb(105, 109, 117)
                };
                ui.painter()
                    .rect_filled(divider_rect, egui::CornerRadius::ZERO, divider_color);

                if let Some(error) = &self.load_error {
                    ui.scope_builder(
                        egui::UiBuilder::new().max_rect(left_panel.shrink(8.0)),
                        |ui| ui.colored_label(Color32::from_rgb(240, 150, 140), error),
                    );
                } else if let Some(contents) = self.gcode_content.as_deref() {
                    let left_scope_rect = left_panel.shrink(8.0);
                    let output = ui.scope_builder(
                        egui::UiBuilder::new().max_rect(left_scope_rect),
                        |ui| {
                            ui.shrink_clip_rect(left_panel);
                            ui.style_mut().spacing.scroll = scrollbar_style;
                            let text_style = egui::TextStyle::Monospace;
                            let row_height = ui.text_style_height(&text_style);
                            let font_id = ui.style().text_styles[&text_style].clone();
                            egui::ScrollArea::horizontal()
                                .id_salt("gcode-left-horizontal")
                                .auto_shrink([false, false])
                                .scroll_bar_visibility(
                                    egui::containers::scroll_area::ScrollBarVisibility::VisibleWhenNeeded,
                                )
                                .show(ui, |ui| {
                                    let glyph_width =
                                        ui.fonts(|fonts| fonts.glyph_width(&font_id, 'M'));
                                    ui.set_min_width(
                                        self.left_max_row_chars as f32 * glyph_width
                                            + ui.spacing().item_spacing.x,
                                    );
                                    egui::ScrollArea::vertical()
                                        .id_salt("gcode-left-vertical")
                                        .auto_shrink([false, false])
                                        .vertical_scroll_offset(self.vertical_scroll_offset)
                                        .scroll_bar_visibility(
                                            egui::containers::scroll_area::ScrollBarVisibility::AlwaysHidden,
                                        )
                                        .show_rows(
                                            ui,
                                            row_height,
                                            self.display_rows.len(),
                                            |ui, rows| {
                                                render_gcode_rows(
                                                    ui,
                                                    contents,
                                                    &self.display_rows,
                                                    rows,
                                                    row_height,
                                                    &self.gcode_translations,
                                                    &self.redlisted_commands,
                                                    false,
                                                );
                                            },
                                        )
                                })
                        },
                    );
                    let horizontal_output = &output.inner;
                    if horizontal_output.content_size.x > horizontal_output.inner_rect.width() {
                        let track_rect = scrollbar_track_rect(
                            horizontal_output.inner_rect,
                            left_scope_rect,
                            horizontal_output.inner_rect,
                            true,
                            scrollbar_style.bar_inner_margin,
                            scrollbar_style.bar_outer_margin,
                        );
                        mask_scrollbar_handle_overflow(
                            ui.painter(),
                            left_panel,
                            track_rect,
                            true,
                            panel_color,
                        );
                        paint_scrollbar(
                            ui,
                            left_panel,
                            track_rect,
                            horizontal_output.inner_rect,
                            true,
                            horizontal_output.state.offset.x,
                            horizontal_output.content_size.x,
                            horizontal_output.inner_rect.width(),
                            scrollbar_style,
                        );
                    }
                    self.vertical_scroll_offset = horizontal_output.inner.state.offset.y;
                }

                if let Some(error) = &self.display_config_error {
                    ui.scope_builder(
                        egui::UiBuilder::new().max_rect(right_panel.shrink(8.0)),
                        |ui| ui.colored_label(Color32::from_rgb(240, 150, 140), error),
                    );
                } else if let Some(contents) = self.gcode_content.as_deref() {
                    let right_scope_rect = right_panel.shrink(8.0);
                    let output = ui.scope_builder(
                        egui::UiBuilder::new().max_rect(right_scope_rect),
                        |ui| {
                            ui.shrink_clip_rect(right_panel);
                            ui.style_mut().spacing.scroll = scrollbar_style;
                            let text_style = egui::TextStyle::Monospace;
                            let row_height = ui.text_style_height(&text_style);
                            let font_id = ui.style().text_styles[&text_style].clone();
                            egui::ScrollArea::both()
                                .id_salt("gcode-right")
                                .auto_shrink([false, false])
                            .vertical_scroll_offset(self.vertical_scroll_offset)
                                .scroll_bar_visibility(
                                    egui::containers::scroll_area::ScrollBarVisibility::VisibleWhenNeeded,
                                )
                            .show_rows(
                                ui,
                                row_height,
                                self.display_rows.len(),
                                |ui, rows| {
                                    let glyph_width =
                                        ui.fonts(|fonts| fonts.glyph_width(&font_id, 'M'));
                                    ui.set_min_width(
                                        self.right_max_row_chars as f32 * glyph_width
                                            + ui.spacing().item_spacing.x,
                                    );
                                    render_gcode_rows(
                                        ui,
                                        contents,
                                        &self.display_rows,
                                        rows,
                                        row_height,
                                        &self.gcode_translations,
                                        &self.redlisted_commands,
                                        true,
                                    );
                                },
                            )
                        },
                    );
                    let scroll_output = &output.inner;
                    let horizontal_track = if scroll_output.content_size.x
                        > scroll_output.inner_rect.width()
                    {
                        Some(scrollbar_track_rect(
                            scroll_output.inner_rect,
                            right_scope_rect,
                            scroll_output.inner_rect,
                            true,
                            scrollbar_style.bar_inner_margin,
                            scrollbar_style.bar_outer_margin,
                        ))
                    } else {
                        None
                    };
                    let vertical_track = if scroll_output.content_size.y
                        > scroll_output.inner_rect.height()
                    {
                        Some(scrollbar_track_rect(
                            scroll_output.inner_rect,
                            right_scope_rect,
                            scroll_output.inner_rect,
                            false,
                            scrollbar_style.bar_inner_margin,
                            scrollbar_style.bar_outer_margin,
                        ))
                    } else {
                        None
                    };
                    if let Some(track_rect) = horizontal_track {
                        mask_scrollbar_handle_overflow(
                            ui.painter(),
                            right_panel,
                            track_rect,
                            true,
                            panel_color,
                        );
                    }
                    if let Some(track_rect) = vertical_track {
                        mask_scrollbar_handle_overflow(
                            ui.painter(),
                            right_panel,
                            track_rect,
                            false,
                            panel_color,
                        );
                    }
                    if let Some(track_rect) = horizontal_track {
                        paint_scrollbar(
                            ui,
                            right_panel,
                            track_rect,
                            scroll_output.inner_rect,
                            true,
                            scroll_output.state.offset.x,
                            scroll_output.content_size.x,
                            scroll_output.inner_rect.width(),
                            scrollbar_style,
                        );
                    }
                    if let Some(track_rect) = vertical_track {
                        paint_scrollbar(
                            ui,
                            right_panel,
                            track_rect,
                            scroll_output.inner_rect,
                            false,
                            scroll_output.state.offset.y,
                            scroll_output.content_size.y,
                            scroll_output.inner_rect.height(),
                            scrollbar_style,
                        );
                    }
                    self.vertical_scroll_offset = scroll_output.state.offset.y;
                }
            });
    }
}

#[cfg(test)]
mod tests {
    use super::{
        gcode_display_lines, load_gcode_redlist, load_gcode_whitelist, load_thumbnail,
        parse_gcode_translations, translate_gcode_line, GcodeDisplayLine,
    };
    use base64::{engine::general_purpose, Engine as _};
    use eframe::egui::Color32;
    use std::collections::HashMap;

    #[test]
    fn loads_thumbnail_from_qoi_block() {
        let mut qoi = b"qoif\0\0\0\x02\0\0\0\x01\x04\0".to_vec();
        qoi.extend_from_slice(&[0xFF, 255, 0, 0, 255, 0x32]);
        qoi.extend_from_slice(b"\0\0\0\0\0\0\0\x01");
        let encoded = general_purpose::STANDARD.encode(qoi);
        let contents = format!("; thumbnail_QOI begin 2x1 28\n; {encoded}\n; thumbnail_QOI end");

        let image = load_thumbnail(&contents).expect("QOI thumbnail should load");
        assert_eq!(image.size, [2, 1]);
        assert_eq!(image.pixels, vec![Color32::RED, Color32::RED]);
    }

    #[test]
    fn loads_qoi_thumbnail_from_sample_gcode() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("Cube_0.2mm_PLA_COREONEL_25m.gcode");
        let contents = std::fs::read_to_string(path).expect("sample G-code should be readable");
        let image = load_thumbnail(&contents).expect("sample QOI thumbnail should load");

        assert_eq!(image.size, [480, 240]);
    }

    #[test]
    fn loads_png_thumbnail_from_cone_slicer_gcode() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("Cone_0.2mm_PLA_Generic Klipper Printer_11m49s.gcode");
        let contents = std::fs::read_to_string(path).expect("Cone G-code should be readable");
        let image = load_thumbnail(&contents).expect("PNG thumbnail should load");

        assert_eq!(image.size, [300, 300]);
    }

    #[test]
    fn loads_whitelist_and_redlist_files() {
        let whitelist = load_gcode_whitelist().expect("G-code whitelist should load");
        let redlist = load_gcode_redlist().expect("G-code redlist should load");

        assert!(whitelist.contains_key("G1"));
        assert_eq!(redlist.get("G5").map(String::as_str), Some("Spline move"));
    }

    #[test]
    fn replaces_thumbnail_sections_and_preserves_original_line_numbers() {
        let contents = "start\n; thumbnail_QOI begin 16x16 376\nqoi data\n; thumbnail_QOI end\nmiddle\n; thumbnail begin 380x285 53116\npng data\n; thumbnail end\n; \"thumbnail\": \"preview.svg\"\nend";

        assert_eq!(
            gcode_display_lines(contents).collect::<Vec<_>>(),
            vec![
                GcodeDisplayLine::FileLine {
                    line_number: 0,
                    text: "start"
                },
                GcodeDisplayLine::ThumbnailSpacer,
                GcodeDisplayLine::ThumbnailHidden,
                GcodeDisplayLine::ThumbnailSpacer,
                GcodeDisplayLine::FileLine {
                    line_number: 4,
                    text: "middle"
                },
                GcodeDisplayLine::ThumbnailSpacer,
                GcodeDisplayLine::ThumbnailHidden,
                GcodeDisplayLine::ThumbnailSpacer,
                GcodeDisplayLine::FileLine {
                    line_number: 8,
                    text: "; \"thumbnail\": \"preview.svg\""
                },
                GcodeDisplayLine::FileLine {
                    line_number: 9,
                    text: "end"
                }
            ]
        );
    }

    #[test]
    fn hides_nested_thumbnail_block_markers_with_one_placeholder() {
        let contents = "before\n; THUMBNAIL_BLOCK_START\n;\n; thumbnail begin 2x2 8\n; data\n; thumbnail end\n; THUMBNAIL_BLOCK_END\nafter";

        assert_eq!(
            gcode_display_lines(contents).collect::<Vec<_>>(),
            vec![
                GcodeDisplayLine::FileLine {
                    line_number: 0,
                    text: "before"
                },
                GcodeDisplayLine::ThumbnailSpacer,
                GcodeDisplayLine::ThumbnailHidden,
                GcodeDisplayLine::ThumbnailSpacer,
                GcodeDisplayLine::FileLine {
                    line_number: 7,
                    text: "after"
                }
            ]
        );
    }

    #[test]
    fn trims_separator_lines_around_consecutive_thumbnail_sections() {
        let contents = "before\n\n;\n; thumbnail begin 2x2 8\n; data\n; thumbnail end\n;\n \n; thumbnail_QOI begin 2x2 8\n; data\n; thumbnail_QOI end\n;\n\n; after";

        assert_eq!(
            gcode_display_lines(contents).collect::<Vec<_>>(),
            vec![
                GcodeDisplayLine::FileLine {
                    line_number: 0,
                    text: "before"
                },
                GcodeDisplayLine::ThumbnailSpacer,
                GcodeDisplayLine::ThumbnailHidden,
                GcodeDisplayLine::ThumbnailSpacer,
                GcodeDisplayLine::FileLine {
                    line_number: 13,
                    text: "; after"
                }
            ]
        );
    }

    #[test]
    fn translates_commands_and_preserves_parameter_values() {
        let translations = parse_gcode_translations(
            "G1: Linear move\nG5: Spline move\nM104: Set hotend temp\nSET_PRESSURE_ADVANCE: Set pressure advance",
        )
        .unwrap();
        let redlisted_commands = HashMap::new();

        let linear_move = translate_gcode_line(
            "G1X1.25 Y2 F1000 ; move",
            &translations,
            &redlisted_commands,
        );
        assert_eq!(linear_move.text, "Linear move X1.25 Y2 F1000");
        assert!(!linear_move.missing_translation);
        assert!(!linear_move.unsupported_command);

        let hotend_temperature =
            translate_gcode_line("N42 M104 S210", &translations, &redlisted_commands);
        assert_eq!(hotend_temperature.text, "Set hotend temp S210");
        assert!(!hotend_temperature.missing_translation);

        let pressure_advance = translate_gcode_line(
            "SET_PRESSURE_ADVANCE ADVANCE=0.036",
            &translations,
            &redlisted_commands,
        );
        assert_eq!(pressure_advance.text, "Set pressure advance ADVANCE=0.036");
        assert!(!pressure_advance.missing_translation);

        let comment = translate_gcode_line("; a comment", &translations, &redlisted_commands);
        assert_eq!(comment.text, "; a comment");
        assert!(!comment.missing_translation);

        let missing = translate_gcode_line("G999 X10", &translations, &redlisted_commands);
        assert_eq!(missing.text, "No translation: G999 X10");
        assert!(missing.missing_translation);
    }

    #[test]
    fn describes_linear_extrusion_and_retraction() {
        let translations = parse_gcode_translations("G1: Linear move").unwrap();
        let redlisted_commands = HashMap::new();

        let extrusion = translate_gcode_line("G1 E0.5 F300", &translations, &redlisted_commands);
        assert_eq!(extrusion.text, "Extrude E0.5 F300");

        let combined_move =
            translate_gcode_line("G1X1.0Y2E0.5", &translations, &redlisted_commands);
        assert_eq!(combined_move.text, "Extrude and linear move X1.0Y2E0.5");

        let retraction = translate_gcode_line("G1 X1 E-0.5", &translations, &redlisted_commands);
        assert_eq!(retraction.text, "Retract X1 E-0.5");
    }

    #[test]
    fn redlisted_commands_are_marked_unsupported_before_translation() {
        let translations = parse_gcode_translations("G5: Spline move").unwrap();
        let redlisted_commands = parse_gcode_translations("G5: Spline move").unwrap();

        let unsupported =
            translate_gcode_line("G5 X1.5 F300 ; spline", &translations, &redlisted_commands);

        assert_eq!(
            unsupported.text,
            "Unsupported command: Spline move X1.5 F300"
        );
        assert!(unsupported.unsupported_command);
        assert!(!unsupported.missing_translation);
    }

    #[test]
    fn rejects_malformed_translation_entries() {
        assert!(parse_gcode_translations("G1 Linear move")
            .unwrap_err()
            .contains("line 1"));
    }
}
