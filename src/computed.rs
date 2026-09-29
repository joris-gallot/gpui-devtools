use gpui::{DivInspectorState, IntoElement, StyleRefinement, div, prelude::*};

use crate::{
  Config,
  box_model::compact_size,
  style_export::optional_debug,
  styles::format_fill,
  ui::{geometry_label, section},
};

#[derive(Debug, PartialEq)]
pub(crate) struct ComputedStyle {
  pub(crate) bounds_size: String,
  pub(crate) content_size: String,
  pub(crate) origin: String,
  pub(crate) explicit_width: String,
  pub(crate) explicit_height: String,
  pub(crate) padding: String,
  pub(crate) margin: String,
  pub(crate) border: String,
  pub(crate) opacity: String,
  pub(crate) visibility: String,
  pub(crate) background: String,
}

pub(crate) fn computed_style(state: &DivInspectorState, style: &StyleRefinement) -> ComputedStyle {
  ComputedStyle {
    bounds_size: compact_size(state.bounds.size),
    content_size: compact_size(state.content_size),
    origin: state.bounds.origin.to_string(),
    explicit_width: optional_debug(style.size.width.as_ref()),
    explicit_height: optional_debug(style.size.height.as_ref()),
    padding: computed_sides_value(&style.padding),
    margin: computed_sides_value(&style.margin),
    border: computed_sides_value(&style.border_widths),
    opacity: optional_debug(style.opacity.as_ref()),
    visibility: optional_debug(style.visibility.as_ref()),
    background: style
      .background
      .as_ref()
      .map(format_fill)
      .unwrap_or_else(|| "auto".into()),
  }
}

pub(crate) fn render_computed(
  state: &DivInspectorState,
  style: &StyleRefinement,
  config: &Config,
) -> impl IntoElement {
  let computed = computed_style(state, style);

  section("Computed", config)
    .id("gpui-devtools-computed")
    .debug_selector(|| "gpui-devtools-computed".into())
    .child(
      div()
        .flex()
        .flex_col()
        .gap_1()
        .text_xs()
        .child(geometry_label("Bounds", computed.bounds_size, config))
        .child(geometry_label("Content", computed.content_size, config))
        .child(geometry_label("Origin", computed.origin, config))
        .child(geometry_label("Width", computed.explicit_width, config))
        .child(geometry_label("Height", computed.explicit_height, config))
        .child(geometry_label("Padding", computed.padding, config))
        .child(geometry_label("Margin", computed.margin, config))
        .child(geometry_label("Border", computed.border, config))
        .child(geometry_label("Opacity", computed.opacity, config))
        .child(geometry_label("Visibility", computed.visibility, config))
        .child(geometry_label("Background", computed.background, config)),
    )
}

fn computed_sides_value<T: Clone + std::fmt::Debug + Default + PartialEq>(
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
      return format!("{top:?}");
    }
    if top == bottom && right == left {
      return format!("{top:?} {right:?}");
    }
  }

  format!(
    "{} {} {} {}",
    top
      .map(|value| format!("{value:?}"))
      .unwrap_or_else(|| "auto".into()),
    right
      .map(|value| format!("{value:?}"))
      .unwrap_or_else(|| "auto".into()),
    bottom
      .map(|value| format!("{value:?}"))
      .unwrap_or_else(|| "auto".into()),
    left
      .map(|value| format!("{value:?}"))
      .unwrap_or_else(|| "auto".into())
  )
}
