use gpui::{Div, DivInspectorState, IntoElement, StyleRefinement, div, prelude::*, rgb};

use crate::{Config, ui::geometry_label};

#[derive(Debug, PartialEq)]
pub(crate) struct BoxModel {
  pub(crate) element_size: String,
  pub(crate) content_size: String,
  pub(crate) content_size_compact: String,
  pub(crate) margin: EdgeValues,
  pub(crate) border: EdgeValues,
  pub(crate) padding: EdgeValues,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EdgeValues {
  pub(crate) top: String,
  pub(crate) right: String,
  pub(crate) bottom: String,
  pub(crate) left: String,
}

pub(crate) fn box_model(state: &DivInspectorState, style: &StyleRefinement) -> BoxModel {
  BoxModel {
    element_size: state.bounds.size.to_string(),
    content_size: state.content_size.to_string(),
    content_size_compact: compact_size(state.content_size),
    margin: edge_values([
      style.margin.top.as_ref(),
      style.margin.right.as_ref(),
      style.margin.bottom.as_ref(),
      style.margin.left.as_ref(),
    ]),
    border: edge_values([
      style.border_widths.top.as_ref(),
      style.border_widths.right.as_ref(),
      style.border_widths.bottom.as_ref(),
      style.border_widths.left.as_ref(),
    ]),
    padding: edge_values([
      style.padding.top.as_ref(),
      style.padding.right.as_ref(),
      style.padding.bottom.as_ref(),
      style.padding.left.as_ref(),
    ]),
  }
}

pub(crate) fn compact_size(size: gpui::Size<gpui::Pixels>) -> String {
  format!(
    "{} x {}",
    compact_pixels(size.width),
    compact_pixels(size.height)
  )
}

pub(crate) fn compact_pixels(pixels: gpui::Pixels) -> String {
  let value = f32::from(pixels);
  if (value.round() - value).abs() < 0.05 {
    return format!("{}", value.round() as i32);
  }

  format!("{value:.1}")
    .trim_end_matches('0')
    .trim_end_matches('.')
    .to_owned()
}

fn edge_values<T: std::fmt::Debug>(sides: [Option<&T>; 4]) -> EdgeValues {
  let [top, right, bottom, left] = sides;
  EdgeValues {
    top: box_side_value(top),
    right: box_side_value(right),
    bottom: box_side_value(bottom),
    left: box_side_value(left),
  }
}

fn box_side_value<T: std::fmt::Debug>(value: Option<&T>) -> String {
  value
    .map(|value| format!("{value:?}"))
    .unwrap_or_else(|| "0px".into())
}

pub(crate) fn render_box_model(
  state: &DivInspectorState,
  style: &StyleRefinement,
  config: &Config,
) -> impl IntoElement {
  let model = box_model(state, style);
  let content = render_content_box(model.content_size_compact, config);
  let padding = render_box_layer(
    "Padding",
    &model.padding,
    content.into_any_element(),
    config,
  );
  let border = render_box_layer("Border", &model.border, padding.into_any_element(), config);
  let margin = render_box_layer("Margin", &model.margin, border.into_any_element(), config);

  div()
    .id("gpui-devtools-box-model")
    .debug_selector(|| "gpui-devtools-box-model".into())
    .p_2()
    .rounded_md()
    .border_1()
    .border_color(rgb(config.accent))
    .bg(rgb(config.background))
    .child(geometry_label("Element", model.element_size, config))
    .child(div().mt_2().child(margin))
}

fn render_content_box(value: String, config: &Config) -> Div {
  div()
    .px_1()
    .py_1()
    .overflow_hidden()
    .rounded_sm()
    .border_1()
    .border_color(rgb(config.border))
    .bg(rgb(config.panel_background))
    .child(
      div()
        .truncate()
        .text_center()
        .text_xs()
        .font_family("monospace")
        .text_color(rgb(config.text))
        .child(value),
    )
}

fn render_box_layer(
  label: &'static str,
  edges: &EdgeValues,
  child: gpui::AnyElement,
  config: &Config,
) -> Div {
  div()
    .p_1()
    .rounded_sm()
    .border_1()
    .border_color(rgb(config.border))
    .bg(rgb(config.background))
    .child(
      div()
        .mb_1()
        .text_xs()
        .text_color(rgb(config.muted_text))
        .child(label),
    )
    .child(edge_value(edges.top.clone(), config))
    .child(
      div()
        .my_1()
        .flex()
        .items_center()
        .gap_1()
        .overflow_hidden()
        .child(edge_value(edges.left.clone(), config))
        .child(div().w_0().flex_1().overflow_hidden().child(child))
        .child(edge_value(edges.right.clone(), config)),
    )
    .child(edge_value(edges.bottom.clone(), config))
}

fn edge_value(value: String, config: &Config) -> Div {
  div()
    .min_w(gpui::px(28.0))
    .text_center()
    .text_xs()
    .font_family("monospace")
    .text_color(rgb(config.text))
    .child(value)
}
