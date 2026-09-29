use std::{cell::RefCell, rc::Rc};

use gpui::{DivInspectorState, InspectorElementId, StyleRefinement};

use crate::{
  box_model::compact_pixels,
  styles::{format_color, format_fill, style_groups},
  temporary_edits::{StyleEdit, StyleEditState, active_style_edits},
};

pub(crate) fn active_style_overrides(
  current: &StyleRefinement,
  original: &StyleRefinement,
  original_size: gpui::Size<gpui::Pixels>,
) -> Vec<StyleEdit> {
  let mut active = Vec::new();

  if current.size.width != original.size.width {
    active.push(StyleEdit::Wider);
  }
  if current.size.height != original.size.height {
    active.push(StyleEdit::Taller);
  }
  if current.padding != original.padding {
    active.push(StyleEdit::Padding);
  }
  if current.margin != original.margin {
    active.push(StyleEdit::Margin);
  }
  if current.border_widths != original.border_widths
    || current.border_color != original.border_color
  {
    active.push(StyleEdit::Border);
  }
  if current.opacity != original.opacity {
    active.push(StyleEdit::HalfOpacity);
  }
  if current.background != original.background {
    active.push(StyleEdit::AccentBackground);
  }
  if current.visibility != original.visibility {
    active.push(StyleEdit::Hide);
  }

  if active.is_empty() {
    active_style_edits(current, original_size)
  } else {
    active
  }
}

pub(crate) fn style_edit_diff_label(
  edit: StyleEdit,
  original: &StyleRefinement,
  current: &StyleRefinement,
  original_size: gpui::Size<gpui::Pixels>,
) -> String {
  format!(
    "{} x",
    style_edit_diff(edit, original, current, original_size)
  )
}

pub(crate) fn style_edit_diff(
  edit: StyleEdit,
  original: &StyleRefinement,
  current: &StyleRefinement,
  original_size: gpui::Size<gpui::Pixels>,
) -> String {
  let (label, before, after) = match edit {
    StyleEdit::Wider | StyleEdit::Narrower => (
      "Width",
      original
        .size
        .width
        .as_ref()
        .map(rust_debug_value)
        .unwrap_or_else(|| compact_pixels(original_size.width)),
      optional_debug(current.size.width.as_ref()),
    ),
    StyleEdit::Taller | StyleEdit::Shorter => (
      "Height",
      original
        .size
        .height
        .as_ref()
        .map(rust_debug_value)
        .unwrap_or_else(|| compact_pixels(original_size.height)),
      optional_debug(current.size.height.as_ref()),
    ),
    StyleEdit::Padding => (
      "Padding",
      style_sides_value(&original.padding),
      style_sides_value(&current.padding),
    ),
    StyleEdit::Margin => (
      "Margin",
      style_sides_value(&original.margin),
      style_sides_value(&current.margin),
    ),
    StyleEdit::Border => (
      "Border",
      style_sides_value(&original.border_widths),
      style_sides_value(&current.border_widths),
    ),
    StyleEdit::HalfOpacity => (
      "Opacity",
      optional_debug(original.opacity.as_ref()),
      optional_debug(current.opacity.as_ref()),
    ),
    StyleEdit::AccentBackground => (
      "Background",
      original
        .background
        .as_ref()
        .map(format_fill)
        .unwrap_or_else(|| "auto".into()),
      current
        .background
        .as_ref()
        .map(format_fill)
        .unwrap_or_else(|| "auto".into()),
    ),
    StyleEdit::Hide => (
      "Visibility",
      optional_debug(original.visibility.as_ref()),
      optional_debug(current.visibility.as_ref()),
    ),
    StyleEdit::Reset => ("Reset", "".into(), "".into()),
  };

  format!("{label}: {before} -> {after}")
}

pub(crate) fn style_sides_value<T: Clone + std::fmt::Debug + Default + PartialEq>(
  sides: &gpui::EdgesRefinement<T>,
) -> String {
  let top = sides.top.as_ref();
  let right = sides.right.as_ref();
  let bottom = sides.bottom.as_ref();
  let left = sides.left.as_ref();

  if top.is_none() && right.is_none() && bottom.is_none() && left.is_none() {
    return "auto".into();
  }

  if let (Some(top), Some(right), Some(bottom), Some(left)) = (top, right, bottom, left) {
    if top == right && top == bottom && top == left {
      return rust_debug_value(top);
    }
    if top == bottom && right == left {
      return format!("{} {}", rust_debug_value(top), rust_debug_value(right));
    }
  }

  format!(
    "{} {} {} {}",
    top.map(rust_debug_value).unwrap_or_else(|| "auto".into()),
    right.map(rust_debug_value).unwrap_or_else(|| "auto".into()),
    bottom
      .map(rust_debug_value)
      .unwrap_or_else(|| "auto".into()),
    left.map(rust_debug_value).unwrap_or_else(|| "auto".into())
  )
}

pub(crate) fn optional_debug<T: std::fmt::Debug>(value: Option<&T>) -> String {
  value
    .map(|value| format!("{value:?}"))
    .unwrap_or_else(|| "auto".into())
}

pub(crate) fn style_summary_with_overrides(
  id: &InspectorElementId,
  state: &DivInspectorState,
  style_edits: &Rc<RefCell<StyleEditState>>,
) -> String {
  let Some((original_style, original_size)) = style_edits.borrow().original_for(id) else {
    return style_summary(&state.base_style);
  };
  let active = active_style_overrides(&state.base_style, &original_style, original_size);
  if active.is_empty() {
    return style_summary(&state.base_style);
  }

  let mut lines = vec!["Active overrides:".to_owned()];
  lines.extend(active.into_iter().map(|edit| {
    format!(
      "  {}",
      style_edit_diff(edit, &original_style, &state.base_style, original_size)
    )
  }));
  lines.push(String::new());
  lines.push("Full style:".into());
  lines.push(style_summary(&state.base_style));
  lines.join("\n")
}

pub(crate) fn style_summary(style: &StyleRefinement) -> String {
  let mut lines = Vec::new();
  for group in style_groups(style) {
    lines.push(format!("{}:", group.label));
    lines.extend(
      group
        .properties
        .into_iter()
        .map(|property| format!("  {}: {}", property.label, property.value)),
    );
  }

  if lines.is_empty() {
    "No explicit style refinements.".into()
  } else {
    lines.join("\n")
  }
}

pub(crate) fn style_rust_snippet(style: &StyleRefinement) -> String {
  let mut lines = vec!["div()".to_owned()];
  push_length_method(&mut lines, "w", style.size.width.as_ref());
  push_length_method(&mut lines, "h", style.size.height.as_ref());
  push_sides_method(&mut lines, "p", &style.padding);
  push_sides_method(&mut lines, "m", &style.margin);
  push_sides_method(&mut lines, "border", &style.border_widths);

  if let Some(border_color) = style.border_color {
    lines.push(format!("  .border_color({})", rust_color(border_color)));
  }
  if let Some(background) = style.background.as_ref() {
    lines.push(format!("  .bg({})", rust_fill(background)));
  }
  if let Some(opacity) = style.opacity {
    lines.push(format!("  .opacity({opacity:?})"));
  }
  if style.visibility == Some(gpui::Visibility::Hidden) {
    lines.push("  .invisible()".into());
  }

  lines.join("\n")
}

fn push_length_method<T: std::fmt::Debug>(
  lines: &mut Vec<String>,
  method: &'static str,
  value: Option<&T>,
) {
  if let Some(value) = value {
    lines.push(format!("  .{method}({})", rust_debug_value(value)));
  }
}

fn push_sides_method<T: Clone + std::fmt::Debug + Default + PartialEq>(
  lines: &mut Vec<String>,
  method: &'static str,
  sides: &gpui::EdgesRefinement<T>,
) {
  let top = sides.top.as_ref();
  let right = sides.right.as_ref();
  let bottom = sides.bottom.as_ref();
  let left = sides.left.as_ref();

  if let (Some(top), Some(right), Some(bottom), Some(left)) = (top, right, bottom, left)
    && top == right
    && top == bottom
    && top == left
  {
    lines.push(format!("  .{method}({})", rust_debug_value(top)));
    return;
  }

  let mut top_emitted = false;
  let mut right_emitted = false;
  let mut bottom_emitted = false;
  let mut left_emitted = false;

  if let (Some(top), Some(bottom)) = (top, bottom)
    && top == bottom
  {
    lines.push(format!(
      "  .{}({})",
      side_method(method, SideMethod::Vertical),
      rust_debug_value(top)
    ));
    top_emitted = true;
    bottom_emitted = true;
  }

  if let (Some(left), Some(right)) = (left, right)
    && left == right
  {
    lines.push(format!(
      "  .{}({})",
      side_method(method, SideMethod::Horizontal),
      rust_debug_value(left)
    ));
    left_emitted = true;
    right_emitted = true;
  }

  for (value, emitted, side) in [
    (top, top_emitted, SideMethod::Top),
    (right, right_emitted, SideMethod::Right),
    (bottom, bottom_emitted, SideMethod::Bottom),
    (left, left_emitted, SideMethod::Left),
  ] {
    if !emitted && let Some(value) = value {
      lines.push(format!(
        "  .{}({})",
        side_method(method, side),
        rust_debug_value(value)
      ));
    }
  }
}

#[derive(Clone, Copy)]
enum SideMethod {
  Top,
  Right,
  Bottom,
  Left,
  Vertical,
  Horizontal,
}

fn side_method(method: &'static str, side: SideMethod) -> &'static str {
  match (method, side) {
    ("p", SideMethod::Top) => "pt",
    ("p", SideMethod::Right) => "pr",
    ("p", SideMethod::Bottom) => "pb",
    ("p", SideMethod::Left) => "pl",
    ("p", SideMethod::Vertical) => "py",
    ("p", SideMethod::Horizontal) => "px",
    ("m", SideMethod::Top) => "mt",
    ("m", SideMethod::Right) => "mr",
    ("m", SideMethod::Bottom) => "mb",
    ("m", SideMethod::Left) => "ml",
    ("m", SideMethod::Vertical) => "my",
    ("m", SideMethod::Horizontal) => "mx",
    ("border", SideMethod::Top) => "border_t",
    ("border", SideMethod::Right) => "border_r",
    ("border", SideMethod::Bottom) => "border_b",
    ("border", SideMethod::Left) => "border_l",
    ("border", SideMethod::Vertical) => "border_y",
    ("border", SideMethod::Horizontal) => "border_x",
    _ => method,
  }
}

fn rust_fill(fill: &gpui::Fill) -> String {
  fill
    .color()
    .and_then(|background| background.as_solid())
    .map(rust_color)
    .unwrap_or_else(|| format!("{fill:?}"))
}

fn rust_color(color: gpui::Hsla) -> String {
  let value = format_color(color)
    .strip_prefix('#')
    .unwrap_or("000000")
    .to_owned();
  format!("rgb(0x{value})")
}

pub(crate) fn rust_debug_value<T: std::fmt::Debug>(value: &T) -> String {
  let value = format!("{value:?}");
  let Some(px_value) = value.strip_suffix("px") else {
    return value;
  };

  let Ok(parsed) = px_value.parse::<f32>() else {
    return value;
  };

  format!("px({parsed:.1})")
}
