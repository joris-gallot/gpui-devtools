use std::{cell::RefCell, rc::Rc};

use gpui::{
  Div, DivInspectorState, InspectorElementId, IntoElement, StyleRefinement, div, prelude::*, rgb,
};

use crate::{
  Config,
  box_model::{compact_pixels, compact_size},
  copy::text_clipboard_item,
  geometry_label, section,
  styles::{format_color, format_fill, style_groups},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StyleEdit {
  Wider,
  Narrower,
  Taller,
  Shorter,
  Padding,
  Margin,
  Border,
  HalfOpacity,
  AccentBackground,
  Hide,
  Reset,
}

#[derive(Debug, Default)]
pub(crate) struct StyleEditState {
  selected: Option<InspectorElementId>,
  original_style: Option<StyleRefinement>,
  original_size: Option<gpui::Size<gpui::Pixels>>,
}

impl StyleEditState {
  pub(crate) fn select(
    &mut self,
    id: InspectorElementId,
    style: &StyleRefinement,
    size: gpui::Size<gpui::Pixels>,
  ) {
    if self.selected.as_ref() == Some(&id) {
      return;
    }

    self.selected = Some(id);
    self.original_style = Some(style.clone());
    self.original_size = Some(size);
  }

  fn original_for(
    &self,
    id: &InspectorElementId,
  ) -> Option<(StyleRefinement, gpui::Size<gpui::Pixels>)> {
    (self.selected.as_ref() == Some(id))
      .then(|| self.original_style.clone().zip(self.original_size))
      .flatten()
  }

  fn is_dirty(&self, id: &InspectorElementId, style: &StyleRefinement) -> bool {
    self
      .original_for(id)
      .is_some_and(|(original, _)| original != *style)
  }
}

pub(crate) fn render_style_edits(
  id: InspectorElementId,
  state: &DivInspectorState,
  style_edits: &Rc<RefCell<StyleEditState>>,
  config: &Config,
) -> Div {
  let is_dirty = style_edits.borrow().is_dirty(&id, &state.base_style);
  let status = if is_dirty {
    "Temporary overrides active. Reset restores the picked style."
  } else {
    "Try quick overrides without changing application code."
  };

  section("Temporary edits", config)
    .child(
      div()
        .text_xs()
        .text_color(rgb(config.muted_text))
        .child(status),
    )
    .child(render_edit_snapshot(state, config))
    .child(render_active_overrides(&id, state, style_edits, config))
    .child(render_edit_group(
      "Layout",
      &[
        ("gpui-devtools-style-wider", "Wider", StyleEdit::Wider),
        (
          "gpui-devtools-style-narrower",
          "Narrower",
          StyleEdit::Narrower,
        ),
        ("gpui-devtools-style-taller", "Taller", StyleEdit::Taller),
        ("gpui-devtools-style-shorter", "Shorter", StyleEdit::Shorter),
      ],
      &id,
      state,
      style_edits,
      config,
    ))
    .child(render_edit_group(
      "Spacing",
      &[
        (
          "gpui-devtools-style-padding",
          "Padding 12",
          StyleEdit::Padding,
        ),
        ("gpui-devtools-style-margin", "Margin 8", StyleEdit::Margin),
        ("gpui-devtools-style-border", "Border 2", StyleEdit::Border),
      ],
      &id,
      state,
      style_edits,
      config,
    ))
    .child(render_edit_group(
      "Appearance",
      &[
        (
          "gpui-devtools-style-opacity",
          "Opacity 50%",
          StyleEdit::HalfOpacity,
        ),
        (
          "gpui-devtools-style-bg",
          "Accent bg",
          StyleEdit::AccentBackground,
        ),
        ("gpui-devtools-style-hide", "Hide", StyleEdit::Hide),
        ("gpui-devtools-style-reset", "Reset", StyleEdit::Reset),
      ],
      &id,
      state,
      style_edits,
      config,
    ))
    .child(render_export_group(&id, state, style_edits, config))
}

fn render_edit_snapshot(state: &DivInspectorState, config: &Config) -> Div {
  div()
    .grid()
    .gap_1()
    .text_xs()
    .child(geometry_label(
      "Size",
      compact_size(state.bounds.size),
      config,
    ))
    .child(geometry_label(
      "Explicit width",
      optional_debug(state.base_style.size.width.as_ref()),
      config,
    ))
    .child(geometry_label(
      "Explicit height",
      optional_debug(state.base_style.size.height.as_ref()),
      config,
    ))
    .child(geometry_label(
      "Opacity",
      optional_debug(state.base_style.opacity.as_ref()),
      config,
    ))
}

fn render_active_overrides(
  id: &InspectorElementId,
  state: &DivInspectorState,
  style_edits: &Rc<RefCell<StyleEditState>>,
  config: &Config,
) -> Div {
  let Some((original_style, original_size)) = style_edits.borrow().original_for(id) else {
    return div();
  };
  let active = active_style_edits(&state.base_style, original_size);
  if active.is_empty() {
    return div();
  }

  div()
    .flex()
    .flex_col()
    .gap_1()
    .child(
      div()
        .text_xs()
        .text_color(rgb(config.muted_text))
        .child("Active overrides"),
    )
    .child(
      div()
        .flex()
        .flex_wrap()
        .gap_2()
        .children(active.into_iter().map(|edit| {
          remove_style_edit_button(
            id.clone(),
            edit,
            &original_style,
            &state.base_style,
            original_size,
            Rc::clone(style_edits),
            config,
          )
        })),
    )
}

fn remove_style_edit_button(
  id: InspectorElementId,
  edit: StyleEdit,
  original_style: &StyleRefinement,
  current_style: &StyleRefinement,
  original_size: gpui::Size<gpui::Pixels>,
  style_edits: Rc<RefCell<StyleEditState>>,
  config: &Config,
) -> impl IntoElement {
  inspector_button(
    style_edit_remove_selector(edit),
    style_edit_diff_label(edit, original_style, current_style, original_size),
    false,
    true,
    config,
  )
  .on_click(move |_, window, cx| {
    let original = style_edits.borrow().original_for(&id);
    let _ = window.with_inspector_state::<DivInspectorState, _>(Some(&id), cx, |state, _| {
      let Some(state) = state else {
        return;
      };
      let Some((original_style, _)) = original else {
        return;
      };

      restore_style_edit(&mut state.base_style, edit, &original_style);
    });
    window.refresh();
  })
}

fn render_edit_group(
  title: &'static str,
  actions: &[(&'static str, &'static str, StyleEdit)],
  id: &InspectorElementId,
  state: &DivInspectorState,
  style_edits: &Rc<RefCell<StyleEditState>>,
  config: &Config,
) -> Div {
  div()
    .flex()
    .flex_col()
    .gap_1()
    .child(
      div()
        .text_xs()
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(rgb(config.accent))
        .child(title),
    )
    .child(render_edit_button_row(
      actions,
      id,
      state,
      style_edits,
      config,
    ))
}

fn render_export_group(
  id: &InspectorElementId,
  state: &DivInspectorState,
  style_edits: &Rc<RefCell<StyleEditState>>,
  config: &Config,
) -> Div {
  div()
    .flex()
    .flex_col()
    .gap_1()
    .child(
      div()
        .text_xs()
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(rgb(config.accent))
        .child("Export"),
    )
    .child(
      div()
        .flex()
        .flex_wrap()
        .gap_2()
        .child(style_copy_button(
          "gpui-devtools-copy-style-summary",
          "Copy summary",
          style_summary_with_overrides(id, state, style_edits),
          config,
        ))
        .child(style_copy_button(
          "gpui-devtools-copy-style-rust",
          "Copy Rust",
          style_rust_snippet(&state.base_style),
          config,
        )),
    )
}

fn render_edit_button_row(
  actions: &[(&'static str, &'static str, StyleEdit)],
  id: &InspectorElementId,
  state: &DivInspectorState,
  style_edits: &Rc<RefCell<StyleEditState>>,
  config: &Config,
) -> Div {
  let original_size = style_edits
    .borrow()
    .original_for(id)
    .map(|(_, size)| size)
    .unwrap_or(state.bounds.size);

  div()
    .flex()
    .flex_wrap()
    .gap_2()
    .children(actions.iter().map(|(selector, label, edit)| {
      style_edit_button(
        selector,
        label,
        *edit,
        id.clone(),
        Rc::clone(style_edits),
        is_style_edit_active(&state.base_style, *edit, original_size),
        config,
      )
    }))
}

fn style_edit_button(
  selector: &'static str,
  label: &'static str,
  edit: StyleEdit,
  id: InspectorElementId,
  style_edits: Rc<RefCell<StyleEditState>>,
  active: bool,
  config: &Config,
) -> impl IntoElement {
  inspector_button(
    selector,
    label.to_owned(),
    edit == StyleEdit::Reset,
    active,
    config,
  )
  .on_click(move |_, window, cx| {
    let original = style_edits.borrow().original_for(&id);
    let _ = window.with_inspector_state::<DivInspectorState, _>(Some(&id), cx, |state, _| {
      let Some(state) = state else {
        return;
      };
      let Some((original_style, original_size)) = original else {
        return;
      };

      if edit == StyleEdit::Reset {
        *state.base_style = original_style;
      } else if is_style_edit_active(&state.base_style, edit, original_size) {
        restore_style_edit(&mut state.base_style, edit, &original_style);
      } else {
        apply_style_edit(&mut state.base_style, edit, original_size);
      }
    });
    window.refresh();
  })
}

fn style_copy_button(
  selector: &'static str,
  label: &'static str,
  value: String,
  config: &Config,
) -> impl IntoElement {
  inspector_button(selector, label.to_owned(), false, false, config).on_click(
    move |_, _window, cx| {
      cx.write_to_clipboard(text_clipboard_item(value.clone()));
    },
  )
}

fn inspector_button(
  selector: &'static str,
  label: String,
  muted: bool,
  active: bool,
  config: &Config,
) -> gpui::Stateful<Div> {
  div()
    .id(selector)
    .debug_selector(|| selector.into())
    .px_2()
    .py_1()
    .rounded_sm()
    .cursor_pointer()
    .border_1()
    .border_color(if active {
      rgb(config.accent)
    } else {
      rgb(config.border)
    })
    .bg(if active {
      rgb(config.accent)
    } else {
      rgb(config.background)
    })
    .text_xs()
    .text_color(if muted {
      rgb(config.muted_text)
    } else {
      rgb(config.text)
    })
    .hover(|button| button.border_color(rgb(config.accent)))
    .child(label)
}

pub(crate) fn apply_style_edit(
  style: &mut StyleRefinement,
  edit: StyleEdit,
  original_size: gpui::Size<gpui::Pixels>,
) {
  match edit {
    StyleEdit::Wider => {
      style.size.width = wide_width(original_size);
    }
    StyleEdit::Narrower => {
      style.size.width = narrow_width(original_size);
    }
    StyleEdit::Taller => {
      style.size.height = tall_height(original_size);
    }
    StyleEdit::Shorter => {
      style.size.height = short_height(original_size);
    }
    StyleEdit::Padding => {
      let padding = edit_padding();
      style.padding.top = Some(padding);
      style.padding.right = Some(padding);
      style.padding.bottom = Some(padding);
      style.padding.left = Some(padding);
    }
    StyleEdit::Margin => {
      let margin = edit_margin();
      style.margin.top = Some(margin);
      style.margin.right = Some(margin);
      style.margin.bottom = Some(margin);
      style.margin.left = Some(margin);
    }
    StyleEdit::Border => {
      let border = edit_border();
      style.border_widths.top = Some(border);
      style.border_widths.right = Some(border);
      style.border_widths.bottom = Some(border);
      style.border_widths.left = Some(border);
      style.border_color = Some(edit_border_color());
    }
    StyleEdit::HalfOpacity => {
      style.opacity = Some(0.5);
    }
    StyleEdit::AccentBackground => {
      style.background = Some(edit_background());
    }
    StyleEdit::Hide => {
      style.visibility = Some(gpui::Visibility::Hidden);
    }
    StyleEdit::Reset => {}
  }
}

pub(crate) fn restore_style_edit(
  style: &mut StyleRefinement,
  edit: StyleEdit,
  original: &StyleRefinement,
) {
  match edit {
    StyleEdit::Wider | StyleEdit::Narrower => {
      style.size.width = original.size.width;
    }
    StyleEdit::Taller | StyleEdit::Shorter => {
      style.size.height = original.size.height;
    }
    StyleEdit::Padding => {
      style.padding = original.padding.clone();
    }
    StyleEdit::Margin => {
      style.margin = original.margin.clone();
    }
    StyleEdit::Border => {
      style.border_widths = original.border_widths.clone();
      style.border_color = original.border_color;
    }
    StyleEdit::HalfOpacity => {
      style.opacity = original.opacity;
    }
    StyleEdit::AccentBackground => {
      style.background = original.background.clone();
    }
    StyleEdit::Hide => {
      style.visibility = original.visibility;
    }
    StyleEdit::Reset => {}
  }
}

pub(crate) fn is_style_edit_active(
  style: &StyleRefinement,
  edit: StyleEdit,
  original_size: gpui::Size<gpui::Pixels>,
) -> bool {
  match edit {
    StyleEdit::Wider => style.size.width == wide_width(original_size),
    StyleEdit::Narrower => style.size.width == narrow_width(original_size),
    StyleEdit::Taller => style.size.height == tall_height(original_size),
    StyleEdit::Shorter => style.size.height == short_height(original_size),
    StyleEdit::Padding => sides_equal(&style.padding, edit_padding()),
    StyleEdit::Margin => sides_equal(&style.margin, edit_margin()),
    StyleEdit::Border => {
      sides_equal(&style.border_widths, edit_border())
        && style.border_color == Some(edit_border_color())
    }
    StyleEdit::HalfOpacity => style.opacity == Some(0.5),
    StyleEdit::AccentBackground => style.background == Some(edit_background()),
    StyleEdit::Hide => style.visibility == Some(gpui::Visibility::Hidden),
    StyleEdit::Reset => false,
  }
}

pub(crate) fn active_style_edits(
  style: &StyleRefinement,
  original_size: gpui::Size<gpui::Pixels>,
) -> Vec<StyleEdit> {
  [
    StyleEdit::Wider,
    StyleEdit::Narrower,
    StyleEdit::Taller,
    StyleEdit::Shorter,
    StyleEdit::Padding,
    StyleEdit::Margin,
    StyleEdit::Border,
    StyleEdit::HalfOpacity,
    StyleEdit::AccentBackground,
    StyleEdit::Hide,
  ]
  .into_iter()
  .filter(|edit| is_style_edit_active(style, *edit, original_size))
  .collect()
}

fn style_edit_remove_selector(edit: StyleEdit) -> &'static str {
  match edit {
    StyleEdit::Wider => "gpui-devtools-remove-style-wider",
    StyleEdit::Narrower => "gpui-devtools-remove-style-narrower",
    StyleEdit::Taller => "gpui-devtools-remove-style-taller",
    StyleEdit::Shorter => "gpui-devtools-remove-style-shorter",
    StyleEdit::Padding => "gpui-devtools-remove-style-padding",
    StyleEdit::Margin => "gpui-devtools-remove-style-margin",
    StyleEdit::Border => "gpui-devtools-remove-style-border",
    StyleEdit::HalfOpacity => "gpui-devtools-remove-style-opacity",
    StyleEdit::AccentBackground => "gpui-devtools-remove-style-bg",
    StyleEdit::Hide => "gpui-devtools-remove-style-hide",
    StyleEdit::Reset => "gpui-devtools-remove-style-reset",
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

fn style_sides_value<T: Clone + std::fmt::Debug + Default + PartialEq>(
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

fn sides_equal<T: Clone + std::fmt::Debug + Default + PartialEq>(
  sides: &gpui::EdgesRefinement<T>,
  value: T,
) -> bool {
  sides.top.as_ref() == Some(&value)
    && sides.right.as_ref() == Some(&value)
    && sides.bottom.as_ref() == Some(&value)
    && sides.left.as_ref() == Some(&value)
}

fn wide_width(size: gpui::Size<gpui::Pixels>) -> Option<gpui::Length> {
  Some(gpui::px(f32::from(size.width) + 40.0).into())
}

fn narrow_width(size: gpui::Size<gpui::Pixels>) -> Option<gpui::Length> {
  Some(gpui::px((f32::from(size.width) - 40.0).max(1.0)).into())
}

fn tall_height(size: gpui::Size<gpui::Pixels>) -> Option<gpui::Length> {
  Some(gpui::px(f32::from(size.height) + 40.0).into())
}

fn short_height(size: gpui::Size<gpui::Pixels>) -> Option<gpui::Length> {
  Some(gpui::px((f32::from(size.height) - 40.0).max(1.0)).into())
}

fn edit_padding() -> gpui::DefiniteLength {
  gpui::px(12.0).into()
}

fn edit_margin() -> gpui::Length {
  gpui::px(8.0).into()
}

fn edit_border() -> gpui::AbsoluteLength {
  gpui::px(2.0).into()
}

fn edit_border_color() -> gpui::Hsla {
  rgb(0x61afef).into()
}

fn edit_background() -> gpui::Fill {
  rgb(0x1f4f73).into()
}

fn optional_debug<T: std::fmt::Debug>(value: Option<&T>) -> String {
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
  let active = active_style_edits(&state.base_style, original_size);
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

fn rust_debug_value<T: std::fmt::Debug>(value: &T) -> String {
  let value = format!("{value:?}");
  let Some(px_value) = value.strip_suffix("px") else {
    return value;
  };

  let Ok(parsed) = px_value.parse::<f32>() else {
    return value;
  };

  format!("px({parsed:.1})")
}
