use std::sync::Arc;

use gpui::{Div, div, img, prelude::*, rgb};

use crate::Config;

pub(crate) fn render_icon(svg: &'static [u8], color: u32) -> gpui::Img {
  img(Arc::new(gpui::Image::from_bytes(
    gpui::ImageFormat::Svg,
    recolor_svg(svg, color),
  )))
}

pub(crate) fn recolor_svg(svg: &[u8], color: u32) -> Vec<u8> {
  let color = format!("#{:06x}", color & 0xffffff);
  String::from_utf8_lossy(svg)
    .replace("currentColor", &color)
    .into_bytes()
}

pub(crate) fn geometry_label(label: &'static str, value: String, config: &Config) -> Div {
  div()
    .flex()
    .items_center()
    .justify_between()
    .gap_2()
    .overflow_hidden()
    .text_xs()
    .child(
      div()
        .w_0()
        .flex_1()
        .truncate()
        .text_color(rgb(config.muted_text))
        .child(label),
    )
    .child(
      div()
        .w_0()
        .flex_1()
        .truncate()
        .font_family("monospace")
        .text_right()
        .child(value),
    )
}

pub(crate) fn section(title: &'static str, config: &Config) -> Div {
  div()
    .p_3()
    .flex()
    .flex_col()
    .gap_2()
    .rounded_md()
    .bg(rgb(config.panel_background))
    .border_1()
    .border_color(rgb(config.border))
    .child(div().font_weight(gpui::FontWeight::SEMIBOLD).child(title))
}

pub(crate) fn property(label: &'static str, value: String, config: &Config) -> Div {
  property_with_action(label, value, None, config)
}

pub(crate) fn property_with_action(
  label: &'static str,
  value: String,
  action: Option<gpui::AnyElement>,
  config: &Config,
) -> Div {
  div()
    .overflow_hidden()
    .flex()
    .flex_col()
    .gap_1()
    .child(
      div()
        .flex()
        .items_center()
        .justify_between()
        .text_xs()
        .text_color(rgb(config.muted_text))
        .child(label)
        .when_some(action, |label, action| label.child(action)),
    )
    .child(
      div()
        .w_full()
        .truncate()
        .text_sm()
        .font_family("monospace")
        .child(value),
    )
}
