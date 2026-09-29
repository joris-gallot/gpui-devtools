use gpui::{Div, StyleRefinement, div, prelude::*, rgb};

use crate::{Config, section};

pub(crate) fn render_styles(style: &StyleRefinement, config: &Config) -> Div {
  let groups = style_groups(style);
  let panel = section("Styles", config);

  if groups.is_empty() {
    panel.child(
      div()
        .text_sm()
        .text_color(rgb(config.muted_text))
        .child("No explicit style refinements."),
    )
  } else {
    panel.children(
      groups
        .into_iter()
        .map(|group| render_style_group(group, config)),
    )
  }
}

fn render_style_group(group: StyleGroup, config: &Config) -> Div {
  div()
    .flex()
    .flex_col()
    .gap_1()
    .child(
      div()
        .pt_1()
        .text_xs()
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(rgb(config.accent))
        .child(group.label),
    )
    .children({
      let count = group.properties.len();
      group
        .properties
        .into_iter()
        .enumerate()
        .map(move |(index, property)| render_style_property(property, index + 1 < count, config))
    })
}

fn render_style_property(property: StyleProperty, show_separator: bool, config: &Config) -> Div {
  div()
    .py_1()
    .flex()
    .items_start()
    .justify_between()
    .gap_3()
    .when(show_separator, |row| {
      row.border_b_1().border_color(rgb(config.border))
    })
    .text_xs()
    .child(
      div()
        .flex_shrink_0()
        .text_color(rgb(config.muted_text))
        .child(property.label),
    )
    .child(
      div()
        .w_0()
        .flex_1()
        .flex()
        .items_center()
        .justify_end()
        .gap_2()
        .when_some(property.swatch, |row, swatch| {
          row.child(
            div()
              .size_3()
              .flex_shrink_0()
              .rounded_sm()
              .border_1()
              .border_color(rgb(config.border))
              .bg(swatch),
          )
        })
        .child(
          div()
            .w_0()
            .flex_1()
            .truncate()
            .font_family("monospace")
            .text_right()
            .child(property.value),
        ),
    )
}

#[derive(Debug, PartialEq)]
pub(crate) struct StyleGroup {
  pub(crate) label: &'static str,
  pub(crate) properties: Vec<StyleProperty>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct StyleProperty {
  pub(crate) label: &'static str,
  pub(crate) value: String,
  pub(crate) swatch: Option<gpui::Fill>,
}

pub(crate) fn style_groups(style: &StyleRefinement) -> Vec<StyleGroup> {
  let mut groups = Vec::new();

  let mut layout = Vec::new();
  push_debug(&mut layout, "Display", style.display.as_ref());
  push_debug(&mut layout, "Visibility", style.visibility.as_ref());
  push_debug(&mut layout, "Overflow X", style.overflow.x.as_ref());
  push_debug(&mut layout, "Overflow Y", style.overflow.y.as_ref());
  push_debug(
    &mut layout,
    "Scrollbar width",
    style.scrollbar_width.as_ref(),
  );
  push_debug(
    &mut layout,
    "Concurrent scroll",
    style.allow_concurrent_scroll.as_ref(),
  );
  push_debug(
    &mut layout,
    "Restrict scroll axis",
    style.restrict_scroll_to_axis.as_ref(),
  );
  push_debug(&mut layout, "Position", style.position.as_ref());
  push_debug(&mut layout, "Inset top", style.inset.top.as_ref());
  push_debug(&mut layout, "Inset right", style.inset.right.as_ref());
  push_debug(&mut layout, "Inset bottom", style.inset.bottom.as_ref());
  push_debug(&mut layout, "Inset left", style.inset.left.as_ref());
  push_debug(&mut layout, "Width", style.size.width.as_ref());
  push_debug(&mut layout, "Height", style.size.height.as_ref());
  push_debug(&mut layout, "Min width", style.min_size.width.as_ref());
  push_debug(&mut layout, "Min height", style.min_size.height.as_ref());
  push_debug(&mut layout, "Max width", style.max_size.width.as_ref());
  push_debug(&mut layout, "Max height", style.max_size.height.as_ref());
  push_debug(&mut layout, "Aspect ratio", style.aspect_ratio.as_ref());
  push_debug(&mut layout, "Align items", style.align_items.as_ref());
  push_debug(&mut layout, "Align self", style.align_self.as_ref());
  push_debug(&mut layout, "Align content", style.align_content.as_ref());
  push_debug(
    &mut layout,
    "Justify content",
    style.justify_content.as_ref(),
  );
  push_debug(&mut layout, "Column gap", style.gap.width.as_ref());
  push_debug(&mut layout, "Row gap", style.gap.height.as_ref());
  push_debug(&mut layout, "Flex direction", style.flex_direction.as_ref());
  push_debug(&mut layout, "Flex wrap", style.flex_wrap.as_ref());
  push_debug(&mut layout, "Flex basis", style.flex_basis.as_ref());
  push_debug(&mut layout, "Flex grow", style.flex_grow.as_ref());
  push_debug(&mut layout, "Flex shrink", style.flex_shrink.as_ref());
  push_debug(&mut layout, "Grid columns", style.grid_cols.as_ref());
  push_debug(&mut layout, "Grid rows", style.grid_rows.as_ref());
  push_debug(&mut layout, "Grid location", style.grid_location.as_ref());
  push_group(&mut groups, "Layout", layout);

  let mut spacing = Vec::new();
  push_compact_sides(
    &mut spacing,
    "Margin",
    [
      ("Margin top", style.margin.top.as_ref()),
      ("Margin right", style.margin.right.as_ref()),
      ("Margin bottom", style.margin.bottom.as_ref()),
      ("Margin left", style.margin.left.as_ref()),
    ],
  );
  push_compact_sides(
    &mut spacing,
    "Padding",
    [
      ("Padding top", style.padding.top.as_ref()),
      ("Padding right", style.padding.right.as_ref()),
      ("Padding bottom", style.padding.bottom.as_ref()),
      ("Padding left", style.padding.left.as_ref()),
    ],
  );
  push_compact_sides(
    &mut spacing,
    "Border",
    [
      ("Border top", style.border_widths.top.as_ref()),
      ("Border right", style.border_widths.right.as_ref()),
      ("Border bottom", style.border_widths.bottom.as_ref()),
      ("Border left", style.border_widths.left.as_ref()),
    ],
  );
  push_group(&mut groups, "Spacing", spacing);

  let mut appearance = Vec::new();
  push_fill(&mut appearance, "Background", style.background.as_ref());
  push_color(&mut appearance, "Border color", style.border_color.as_ref());
  push_debug(&mut appearance, "Border style", style.border_style.as_ref());
  push_compact_sides(
    &mut appearance,
    "Radius",
    [
      ("Radius top left", style.corner_radii.top_left.as_ref()),
      ("Radius top right", style.corner_radii.top_right.as_ref()),
      (
        "Radius bottom right",
        style.corner_radii.bottom_right.as_ref(),
      ),
      (
        "Radius bottom left",
        style.corner_radii.bottom_left.as_ref(),
      ),
    ],
  );
  push_shadows(&mut appearance, "Box shadow", style.box_shadow.as_ref());
  push_debug(&mut appearance, "Cursor", style.mouse_cursor.as_ref());
  push_debug(&mut appearance, "Opacity", style.opacity.as_ref());
  push_group(&mut groups, "Appearance", appearance);

  if let Some(text) = style.text.explicit_refinement() {
    let mut typography = Vec::new();
    push_color(&mut typography, "Color", text.color.as_ref());
    push_text(&mut typography, "Font family", text.font_family.as_ref());
    push_font_features(&mut typography, text.font_features.as_ref());
    push_font_fallbacks(&mut typography, text.font_fallbacks.as_ref());
    push_debug(&mut typography, "Font size", text.font_size.as_ref());
    push_debug(&mut typography, "Line height", text.line_height.as_ref());
    push_debug(&mut typography, "Font weight", text.font_weight.as_ref());
    push_debug(&mut typography, "Font style", text.font_style.as_ref());
    push_color(
      &mut typography,
      "Background",
      text.background_color.as_ref(),
    );
    push_underline(&mut typography, text.underline.as_ref());
    push_strikethrough(&mut typography, text.strikethrough.as_ref());
    push_debug(&mut typography, "White space", text.white_space.as_ref());
    push_text_overflow(&mut typography, text.text_overflow.as_ref());
    push_debug(&mut typography, "Text align", text.text_align.as_ref());
    push_debug(&mut typography, "Line clamp", text.line_clamp.as_ref());
    push_group(&mut groups, "Typography", typography);
  }

  groups
}

trait ExplicitTextRefinement {
  fn explicit_refinement(&self) -> Option<&gpui::TextStyleRefinement>;
}

impl ExplicitTextRefinement for gpui::TextStyleRefinement {
  fn explicit_refinement(&self) -> Option<&gpui::TextStyleRefinement> {
    self.is_some().then_some(self)
  }
}

impl ExplicitTextRefinement for Option<gpui::TextStyleRefinement> {
  fn explicit_refinement(&self) -> Option<&gpui::TextStyleRefinement> {
    self.as_ref().filter(|text| text.is_some())
  }
}

pub(crate) fn push_shadows(
  properties: &mut Vec<StyleProperty>,
  label: &'static str,
  shadows: Option<&Vec<gpui::BoxShadow>>,
) {
  let Some(shadows) = shadows.filter(|shadows| !shadows.is_empty()) else {
    return;
  };

  push_value(
    properties,
    label,
    shadows
      .iter()
      .map(format_shadow)
      .collect::<Vec<_>>()
      .join(", "),
  );
}

pub(crate) fn format_shadow(shadow: &gpui::BoxShadow) -> String {
  let inset = if shadow.inset { "inset " } else { "" };
  format!(
    "{inset}{} {} {} {} {}",
    shadow.offset.x,
    shadow.offset.y,
    shadow.blur_radius,
    shadow.spread_radius,
    format_color(shadow.color)
  )
}

pub(crate) fn push_underline(
  properties: &mut Vec<StyleProperty>,
  underline: Option<&gpui::UnderlineStyle>,
) {
  if let Some(underline) = underline {
    let wavy = if underline.wavy { "wavy " } else { "" };
    push_value(
      properties,
      "Underline",
      match underline.color {
        Some(color) => format!("{wavy}{} {}", underline.thickness, format_color(color)),
        None => format!("{wavy}{}", underline.thickness),
      },
    );
  }
}

pub(crate) fn push_strikethrough(
  properties: &mut Vec<StyleProperty>,
  strikethrough: Option<&gpui::StrikethroughStyle>,
) {
  if let Some(strikethrough) = strikethrough {
    push_value(
      properties,
      "Strikethrough",
      match strikethrough.color {
        Some(color) => format!("{} {}", strikethrough.thickness, format_color(color)),
        None => strikethrough.thickness.to_string(),
      },
    );
  }
}

pub(crate) fn push_text_overflow(
  properties: &mut Vec<StyleProperty>,
  overflow: Option<&gpui::TextOverflow>,
) {
  if let Some(overflow) = overflow {
    let (position, ellipsis) = match overflow {
      gpui::TextOverflow::Truncate(ellipsis) => ("Truncate end", ellipsis),
      gpui::TextOverflow::TruncateStart(ellipsis) => ("Truncate start", ellipsis),
      gpui::TextOverflow::TruncateMiddle(ellipsis) => ("Truncate middle", ellipsis),
    };
    push_value(
      properties,
      "Text overflow",
      format!("{position} {ellipsis}"),
    );
  }
}

pub(crate) fn push_font_features(
  properties: &mut Vec<StyleProperty>,
  features: Option<&gpui::FontFeatures>,
) {
  let Some(features) = features.filter(|features| !features.0.is_empty()) else {
    return;
  };

  push_value(
    properties,
    "Font features",
    features
      .0
      .iter()
      .map(|(feature, value)| format!("{feature} {value}"))
      .collect::<Vec<_>>()
      .join(", "),
  );
}

pub(crate) fn push_font_fallbacks(
  properties: &mut Vec<StyleProperty>,
  fallbacks: Option<&gpui::FontFallbacks>,
) {
  let Some(fallbacks) = fallbacks.filter(|fallbacks| !fallbacks.0.is_empty()) else {
    return;
  };

  push_value(properties, "Font fallbacks", fallbacks.0.join(", "));
}

pub(crate) fn push_text(
  properties: &mut Vec<StyleProperty>,
  label: &'static str,
  value: Option<&gpui::SharedString>,
) {
  if let Some(value) = value {
    push_value(properties, label, value.to_string());
  }
}

pub(crate) fn push_color(
  properties: &mut Vec<StyleProperty>,
  label: &'static str,
  color: Option<&gpui::Hsla>,
) {
  if let Some(color) = color {
    properties.push(StyleProperty {
      label,
      value: format_color(*color),
      swatch: Some((*color).into()),
    });
  }
}

pub(crate) fn push_fill(
  properties: &mut Vec<StyleProperty>,
  label: &'static str,
  fill: Option<&gpui::Fill>,
) {
  if let Some(fill) = fill {
    properties.push(StyleProperty {
      label,
      value: format_fill(fill),
      swatch: Some(fill.clone()),
    });
  }
}

pub(crate) fn format_fill(fill: &gpui::Fill) -> String {
  match fill.color().and_then(|background| background.as_solid()) {
    Some(color) => format_color(color),
    None => match fill {
      gpui::Fill::Color(background) => format!("{background:?}"),
    },
  }
}

pub(crate) fn format_color(color: gpui::Hsla) -> String {
  let rgba = color.to_rgb();
  let [red, green, blue, alpha] = [rgba.r, rgba.g, rgba.b, rgba.a]
    .map(|component| (component.clamp(0.0, 1.0) * 255.0).round() as u8);

  if alpha == u8::MAX {
    format!("#{red:02x}{green:02x}{blue:02x}")
  } else {
    format!("#{red:02x}{green:02x}{blue:02x}{alpha:02x}")
  }
}

pub(crate) fn push_compact_sides<T: std::fmt::Debug + PartialEq>(
  properties: &mut Vec<StyleProperty>,
  label: &'static str,
  sides: [(&'static str, Option<&T>); 4],
) {
  let [
    (top_label, top),
    (right_label, right),
    (bottom_label, bottom),
    (left_label, left),
  ] = sides;

  if let (Some(top), Some(right), Some(bottom), Some(left)) = (top, right, bottom, left) {
    if top == right && top == bottom && top == left {
      push_value(properties, label, format!("{top:?}"));
      return;
    }
    if top == bottom && right == left {
      push_value(properties, label, format!("{top:?} {right:?}"));
      return;
    }
  }

  push_debug(properties, top_label, top);
  push_debug(properties, right_label, right);
  push_debug(properties, bottom_label, bottom);
  push_debug(properties, left_label, left);
}

pub(crate) fn push_value(properties: &mut Vec<StyleProperty>, label: &'static str, value: String) {
  properties.push(StyleProperty {
    label,
    value,
    swatch: None,
  });
}

pub(crate) fn push_debug<T: std::fmt::Debug>(
  properties: &mut Vec<StyleProperty>,
  label: &'static str,
  value: Option<&T>,
) {
  if let Some(value) = value {
    properties.push(StyleProperty {
      label,
      value: format!("{value:?}"),
      swatch: None,
    });
  }
}

pub(crate) fn push_group(
  groups: &mut Vec<StyleGroup>,
  label: &'static str,
  properties: Vec<StyleProperty>,
) {
  if !properties.is_empty() {
    groups.push(StyleGroup { label, properties });
  }
}
