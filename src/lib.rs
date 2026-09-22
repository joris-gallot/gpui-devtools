//! Developer tools for inspecting and debugging [GPUI](https://gpui.rs) applications.
//!
//! The inspector renders next to the application window and shows the picked element's source
//! location, GPUI element ID, bounds, content size and `Div` style refinements.
//!
//! Install it once the rest of the application is initialized, after any library that registers
//! its own inspector renderer:
//!
//! ```
//! # use gpui::App;
//! fn setup_devtools(cx: &mut App) {
//!   gpui_devtools::init(cx);
//! }
//! ```
//!
//! This binds [`ToggleInspector`] to `cmd-alt-i` on macOS and `ctrl-alt-i` elsewhere. Use
//! [`init_with`] with a [`Config`] to change the key binding or the panel colors.
//!
//! GPUI only builds its inspector with the `inspector` feature or in debug builds, so gate this
//! crate behind a feature of your own to keep it out of release builds.

#![warn(missing_docs)]

use std::{cell::RefCell, rc::Rc, sync::Arc, time::Duration};

use gpui::{
  App, ClipboardItem, Context, Div, DivInspectorState, Inspector, InspectorElementId, IntoElement,
  KeyBinding, StyleRefinement, Window, actions, div, img, prelude::*, rgb,
};

const DEFAULT_MACOS_KEY_BINDING: &str = "cmd-alt-i";
const DEFAULT_OTHER_KEY_BINDING: &str = "ctrl-alt-i";
const COPY_FEEDBACK_DURATION: Duration = Duration::from_millis(1500);
const PICK_ICON_SVG: &[u8] = include_bytes!("../assets/icons/square-dashed-mouse-pointer.svg");
const CLOSE_ICON_SVG: &[u8] = include_bytes!("../assets/icons/x.svg");

actions!(
  gpui_devtools,
  [
    /// Shows or hides the inspector in the active window.
    ToggleInspector
  ]
);

/// Installation options for the inspector.
///
/// Colors are `0xRRGGBB` values, matching [`gpui::rgb`].
#[derive(Clone, Debug)]
pub struct Config {
  /// Key binding for [`ToggleInspector`], or `None` to register none.
  ///
  /// Defaults to `cmd-alt-i` on macOS and `ctrl-alt-i` elsewhere.
  pub key_binding: Option<&'static str>,
  /// Background of the inspector panel.
  pub background: u32,
  /// Background of sections and controls inside the panel.
  pub panel_background: u32,
  /// Color of borders and separators.
  pub border: u32,
  /// Color of primary text.
  pub text: u32,
  /// Color of labels and secondary text.
  pub muted_text: u32,
  /// Color of highlights, such as the active picker and group headings.
  pub accent: u32,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      key_binding: Some(default_key_binding()),
      background: 0x111318,
      panel_background: 0x191c22,
      border: 0x30343d,
      text: 0xe6e9ef,
      muted_text: 0x9299a8,
      accent: 0x61afef,
    }
  }
}

impl Config {
  /// Sets the key binding for [`ToggleInspector`], or removes it with `None`.
  ///
  /// ```
  /// let config = gpui_devtools::Config::default().key_binding(Some("ctrl-shift-i"));
  /// ```
  pub fn key_binding(mut self, key_binding: Option<&'static str>) -> Self {
    self.key_binding = key_binding;
    self
  }
}

/// Installs the inspector with the default configuration.
///
/// See [`init_with`] to customize it.
pub fn init(cx: &mut App) {
  init_with(Config::default(), cx);
}

/// Installs the inspector with the given configuration.
///
/// This binds the configured key, handles [`ToggleInspector`], and registers the inspector
/// renderer. Call it after libraries that register their own renderer, since GPUI keeps only the
/// last one.
///
/// ```
/// # use gpui::App;
/// fn setup_devtools(cx: &mut App) {
///   gpui_devtools::init_with(
///     gpui_devtools::Config::default().key_binding(None),
///     cx,
///   );
/// }
/// ```
pub fn init_with(config: Config, cx: &mut App) {
  if let Some(key_binding) = config.key_binding {
    cx.bind_keys([KeyBinding::new(key_binding, ToggleInspector, None)]);
  }

  cx.on_action(|_: &ToggleInspector, cx| toggle_active_window(cx));

  let div_config = config.clone();
  cx.register_inspector_element(move |_window, _cx| {
    let div_config = div_config.clone();
    move |_id, state: &DivInspectorState, _window: &mut Window, _cx: &mut App| {
      render_div_state(state, &div_config)
    }
  });

  let copy_feedback = Rc::new(RefCell::new(CopyFeedback::default()));
  cx.set_inspector_renderer(Box::new(move |inspector, window, cx| {
    render_inspector(inspector, window, cx, &copy_feedback, &config).into_any_element()
  }));
}

/// Shows or hides the inspector in the active window.
///
/// Does nothing when no window is active. Dispatching [`ToggleInspector`] calls this, so you only
/// need it to drive the inspector from your own code.
pub fn toggle_active_window(cx: &mut App) {
  let Some(active_window) = cx.active_window() else {
    return;
  };

  cx.defer(move |cx| {
    let _ = active_window.update(cx, |_, window, cx| window.toggle_inspector(cx));
  });
}

fn render_inspector(
  inspector: &mut Inspector,
  window: &mut Window,
  cx: &mut Context<Inspector>,
  copy_feedback: &Rc<RefCell<CopyFeedback>>,
  config: &Config,
) -> Div {
  let active_element = inspector.active_element_id().cloned();
  let is_picking = inspector.is_picking();
  let inspector_states = inspector.render_inspector_states(window, cx);
  let content = div()
    .id("gpui-devtools-content")
    .flex_1()
    .overflow_y_scroll()
    .p_3()
    .flex()
    .flex_col()
    .gap_3();
  let content = if let Some(id) = active_element {
    content
      .child(render_element_id(&id, cx, copy_feedback, config))
      .children(inspector_states)
  } else {
    content.child(render_empty_state(is_picking, config))
  };

  let pick_button = div()
    .id("gpui-devtools-pick")
    .debug_selector(|| "gpui-devtools-pick".into())
    .size(gpui::px(28.0))
    .flex()
    .items_center()
    .justify_center()
    .rounded_md()
    .cursor_pointer()
    .border_1()
    .border_color(if is_picking {
      rgb(config.accent)
    } else {
      rgb(config.border)
    })
    .bg(if is_picking {
      rgb(config.accent)
    } else {
      rgb(config.panel_background)
    })
    .hover(|button| button.border_color(rgb(config.accent)))
    .child(render_icon(PICK_ICON_SVG, config.text).size_4())
    .on_click(cx.listener(|inspector, _, window, _cx| {
      inspector.start_picking();
      window.refresh();
    }));
  let close_button = div()
    .id("gpui-devtools-close")
    .debug_selector(|| "gpui-devtools-close".into())
    .size(gpui::px(28.0))
    .flex()
    .items_center()
    .justify_center()
    .rounded_md()
    .cursor_pointer()
    .hover(|button| button.bg(rgb(config.panel_background)))
    .child(render_icon(CLOSE_ICON_SVG, config.muted_text).size_4())
    .on_click(cx.listener(|_inspector, _, window, cx| {
      window.toggle_inspector(cx);
    }));

  div()
    .size_full()
    .flex()
    .flex_col()
    .bg(rgb(config.background))
    .text_color(rgb(config.text))
    .border_l_1()
    .border_color(rgb(config.border))
    .child(
      div()
        .h_12()
        .px_3()
        .flex()
        .items_center()
        .justify_between()
        .border_b_1()
        .border_color(rgb(config.border))
        .child(
          div()
            .flex()
            .items_center()
            .gap_2()
            .child(pick_button)
            .child(
              div()
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .child("GPUI DevTools"),
            ),
        )
        .child(close_button),
    )
    .child(content)
}

fn render_icon(svg: &'static [u8], color: u32) -> gpui::Img {
  img(Arc::new(gpui::Image::from_bytes(
    gpui::ImageFormat::Svg,
    recolor_svg(svg, color),
  )))
}

fn recolor_svg(svg: &[u8], color: u32) -> Vec<u8> {
  let color = format!("#{:06x}", color & 0xffffff);
  String::from_utf8_lossy(svg)
    .replace("currentColor", &color)
    .into_bytes()
}

fn render_empty_state(is_picking: bool, config: &Config) -> Div {
  let (title, description) = empty_state_copy(is_picking);

  div()
    .flex_1()
    .py_12()
    .px_4()
    .flex()
    .flex_col()
    .items_center()
    .justify_center()
    .gap_2()
    .text_center()
    .child(
      div()
        .text_lg()
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .child(title),
    )
    .child(
      div()
        .text_sm()
        .text_color(rgb(config.muted_text))
        .child(description),
    )
}

fn empty_state_copy(is_picking: bool) -> (&'static str, &'static str) {
  if is_picking {
    (
      "Pick an element",
      "Move over the application and click to inspect.",
    )
  } else {
    (
      "No element selected",
      "Use Pick to select an element in the application.",
    )
  }
}

fn render_element_id(
  id: &InspectorElementId,
  cx: &mut Context<Inspector>,
  copy_feedback: &Rc<RefCell<CopyFeedback>>,
  config: &Config,
) -> Div {
  let source = source_location(id);
  let global_id = id.path.global_id.to_string();

  section("Selected element", config)
    .child(copyable_property(
      CopyableProperty {
        id: "gpui-devtools-copy-source",
        label: "Source",
        display_value: source.clone(),
        copy_value: source,
        target: CopyTarget::Source,
      },
      cx,
      copy_feedback,
      config,
    ))
    .child(
      div()
        .flex()
        .gap_3()
        .child(property("Instance", id.instance_id.to_string(), config))
        .child(
          copyable_property(
            CopyableProperty {
              id: "gpui-devtools-copy-global-id",
              label: "Global ID",
              display_value: truncate_middle(&global_id, 48),
              copy_value: global_id,
              target: CopyTarget::GlobalId,
            },
            cx,
            copy_feedback,
            config,
          )
          .w_0()
          .flex_1(),
        ),
    )
}

fn render_div_state(state: &DivInspectorState, config: &Config) -> Div {
  div()
    .flex()
    .flex_col()
    .gap_3()
    .child(
      section("Box Model", config)
        .child(render_box_model(state, &state.base_style, config))
        .child(property("Origin", state.bounds.origin.to_string(), config)),
    )
    .child(render_styles(&state.base_style, config))
}

fn render_styles(style: &StyleRefinement, config: &Config) -> Div {
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
struct StyleGroup {
  label: &'static str,
  properties: Vec<StyleProperty>,
}

#[derive(Debug, PartialEq)]
struct StyleProperty {
  label: &'static str,
  value: String,
  swatch: Option<gpui::Fill>,
}

#[derive(Debug, PartialEq)]
struct BoxModel {
  element_size: String,
  content_size: String,
  content_size_compact: String,
  margin: EdgeValues,
  border: EdgeValues,
  padding: EdgeValues,
}

#[derive(Clone, Debug, PartialEq)]
struct EdgeValues {
  top: String,
  right: String,
  bottom: String,
  left: String,
}

fn box_model(state: &DivInspectorState, style: &StyleRefinement) -> BoxModel {
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

fn compact_size(size: gpui::Size<gpui::Pixels>) -> String {
  format!(
    "{} x {}",
    compact_pixels(size.width),
    compact_pixels(size.height)
  )
}

fn compact_pixels(pixels: gpui::Pixels) -> String {
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

fn style_groups(style: &StyleRefinement) -> Vec<StyleGroup> {
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

fn push_shadows(
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

fn format_shadow(shadow: &gpui::BoxShadow) -> String {
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

fn push_underline(properties: &mut Vec<StyleProperty>, underline: Option<&gpui::UnderlineStyle>) {
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

fn push_strikethrough(
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

fn push_text_overflow(properties: &mut Vec<StyleProperty>, overflow: Option<&gpui::TextOverflow>) {
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

fn push_font_features(properties: &mut Vec<StyleProperty>, features: Option<&gpui::FontFeatures>) {
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

fn push_font_fallbacks(
  properties: &mut Vec<StyleProperty>,
  fallbacks: Option<&gpui::FontFallbacks>,
) {
  let Some(fallbacks) = fallbacks.filter(|fallbacks| !fallbacks.0.is_empty()) else {
    return;
  };

  push_value(properties, "Font fallbacks", fallbacks.0.join(", "));
}

fn push_text(
  properties: &mut Vec<StyleProperty>,
  label: &'static str,
  value: Option<&gpui::SharedString>,
) {
  if let Some(value) = value {
    push_value(properties, label, value.to_string());
  }
}

fn push_color(
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

fn push_fill(properties: &mut Vec<StyleProperty>, label: &'static str, fill: Option<&gpui::Fill>) {
  if let Some(fill) = fill {
    properties.push(StyleProperty {
      label,
      value: format_fill(fill),
      swatch: Some(fill.clone()),
    });
  }
}

fn format_fill(fill: &gpui::Fill) -> String {
  match fill.color().and_then(|background| background.as_solid()) {
    Some(color) => format_color(color),
    None => match fill {
      gpui::Fill::Color(background) => format!("{background:?}"),
    },
  }
}

fn format_color(color: gpui::Hsla) -> String {
  let rgba = color.to_rgb();
  let [red, green, blue, alpha] = [rgba.r, rgba.g, rgba.b, rgba.a]
    .map(|component| (component.clamp(0.0, 1.0) * 255.0).round() as u8);

  if alpha == u8::MAX {
    format!("#{red:02x}{green:02x}{blue:02x}")
  } else {
    format!("#{red:02x}{green:02x}{blue:02x}{alpha:02x}")
  }
}

fn push_compact_sides<T: std::fmt::Debug + PartialEq>(
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

fn push_value(properties: &mut Vec<StyleProperty>, label: &'static str, value: String) {
  properties.push(StyleProperty {
    label,
    value,
    swatch: None,
  });
}

fn push_debug<T: std::fmt::Debug>(
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

fn push_group(groups: &mut Vec<StyleGroup>, label: &'static str, properties: Vec<StyleProperty>) {
  if !properties.is_empty() {
    groups.push(StyleGroup { label, properties });
  }
}

fn render_box_model(
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

fn geometry_label(label: &'static str, value: String, config: &Config) -> Div {
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

fn section(title: &'static str, config: &Config) -> Div {
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

fn property(label: &'static str, value: String, config: &Config) -> Div {
  property_with_action(label, value, None, config)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CopyTarget {
  Source,
  GlobalId,
}

#[derive(Debug, Default)]
struct CopyFeedback {
  copied: Option<(CopyTarget, String)>,
  generation: u64,
}

impl CopyFeedback {
  fn is_copied(&self, target: CopyTarget, value: &str) -> bool {
    self
      .copied
      .as_ref()
      .is_some_and(|copied| copied.0 == target && copied.1 == value)
  }

  fn mark_copied(&mut self, target: CopyTarget, value: String) -> u64 {
    self.generation = self.generation.wrapping_add(1);
    self.copied = Some((target, value));
    self.generation
  }

  fn clear(&mut self, generation: u64) -> bool {
    if self.generation != generation {
      return false;
    }

    self.copied = None;
    true
  }
}

struct CopyableProperty {
  id: &'static str,
  label: &'static str,
  display_value: String,
  copy_value: String,
  target: CopyTarget,
}

fn copyable_property(
  property: CopyableProperty,
  cx: &mut Context<Inspector>,
  copy_feedback: &Rc<RefCell<CopyFeedback>>,
  config: &Config,
) -> Div {
  let CopyableProperty {
    id,
    label,
    display_value,
    copy_value,
    target,
  } = property;
  let is_copied = copy_feedback.borrow().is_copied(target, &copy_value);
  let copy_feedback = Rc::clone(copy_feedback);
  let action = div()
    .id(id)
    .debug_selector(|| id.into())
    .w(gpui::px(56.0))
    .px_1()
    .rounded_sm()
    .cursor_pointer()
    .text_center()
    .text_xs()
    .whitespace_nowrap()
    .text_color(rgb(config.accent))
    .hover(|button| button.bg(rgb(config.background)))
    .child(if is_copied { "Copied!" } else { "Copy" })
    .on_click(cx.listener(move |_inspector, _, window, cx| {
      cx.write_to_clipboard(text_clipboard_item(copy_value.clone()));
      let generation = copy_feedback
        .borrow_mut()
        .mark_copied(target, copy_value.clone());
      window.refresh();

      let copy_feedback = Rc::clone(&copy_feedback);
      cx.spawn(async move |inspector, cx| {
        cx.background_executor().timer(COPY_FEEDBACK_DURATION).await;
        let cleared = copy_feedback.borrow_mut().clear(generation);
        if cleared {
          let _ = inspector.update(cx, |_, cx| cx.notify());
        }
      })
      .detach();
    }));

  property_with_action(
    label,
    display_value,
    Some(action.into_any_element()),
    config,
  )
}

fn text_clipboard_item(value: String) -> ClipboardItem {
  ClipboardItem::new_string(value)
}

fn property_with_action(
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

fn truncate_middle(value: &str, max_chars: usize) -> String {
  let chars = value.chars().collect::<Vec<_>>();
  if chars.len() <= max_chars {
    return value.to_owned();
  }
  if max_chars <= 1 {
    return "…".chars().take(max_chars).collect();
  }

  let available = max_chars - 1;
  let start = available.div_ceil(2);
  let end = available - start;
  format!(
    "{}…{}",
    chars[..start].iter().collect::<String>(),
    chars[chars.len() - end..].iter().collect::<String>()
  )
}

fn source_location(id: &InspectorElementId) -> String {
  let location = id.path.source_location;
  format!(
    "{}:{}:{}",
    compact_source_path(location.file()),
    location.line(),
    location.column()
  )
}

fn compact_source_path(file: &str) -> String {
  let components = std::path::Path::new(file)
    .components()
    .filter_map(|component| match component {
      std::path::Component::Normal(component) => component.to_str(),
      _ => None,
    })
    .collect::<Vec<_>>();
  let start = components
    .iter()
    .rposition(|component| *component == "src")
    .unwrap_or_else(|| components.len().saturating_sub(1));
  components[start..].join("/")
}

const fn default_key_binding() -> &'static str {
  if cfg!(target_os = "macos") {
    DEFAULT_MACOS_KEY_BINDING
  } else {
    DEFAULT_OTHER_KEY_BINDING
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn config_can_disable_the_default_key_binding() {
    assert_eq!(Config::default().key_binding(None).key_binding, None);
  }

  #[test]
  fn default_key_binding_matches_the_platform() {
    let expected = if cfg!(target_os = "macos") {
      "cmd-alt-i"
    } else {
      "ctrl-alt-i"
    };
    assert_eq!(default_key_binding(), expected);
  }

  #[test]
  fn empty_state_copy_matches_picker_state() {
    assert_eq!(empty_state_copy(true).0, "Pick an element");
    assert_eq!(empty_state_copy(false).0, "No element selected");
  }

  #[test]
  fn equal_and_opposite_sides_are_compacted() {
    let mut properties = Vec::new();
    push_compact_sides(
      &mut properties,
      "Padding",
      [
        ("Padding top", Some(&1)),
        ("Padding right", Some(&2)),
        ("Padding bottom", Some(&1)),
        ("Padding left", Some(&2)),
      ],
    );
    assert_eq!(
      properties,
      vec![StyleProperty {
        label: "Padding",
        value: "1 2".into(),
        swatch: None,
      }]
    );

    properties.clear();
    push_compact_sides(
      &mut properties,
      "Border",
      [
        ("Border top", Some(&1)),
        ("Border right", Some(&1)),
        ("Border bottom", Some(&1)),
        ("Border left", Some(&1)),
      ],
    );
    assert_eq!(properties[0].value, "1");
  }

  #[test]
  fn style_groups_only_include_explicit_refinements() {
    let mut style = StyleRefinement::default();
    assert!(style_groups(&style).is_empty());

    style.opacity = Some(0.5);
    assert_eq!(
      style_groups(&style),
      vec![StyleGroup {
        label: "Appearance",
        properties: vec![StyleProperty {
          label: "Opacity",
          value: "0.5".into(),
          swatch: None,
        }],
      }]
    );
  }

  #[test]
  fn box_model_includes_layout_metrics_and_spacing() {
    let mut style = StyleRefinement::default();
    style.margin.top = Some(gpui::px(1.0).into());
    style.margin.right = Some(gpui::px(2.0).into());
    style.padding.top = Some(gpui::px(3.0).into());
    style.padding.right = Some(gpui::px(4.0).into());
    style.padding.bottom = Some(gpui::px(5.0).into());
    style.padding.left = Some(gpui::px(6.0).into());
    style.border_widths.top = Some(gpui::px(7.0).into());

    let state = DivInspectorState {
      base_style: Box::new(style.clone()),
      bounds: gpui::Bounds {
        origin: gpui::point(gpui::px(10.0), gpui::px(20.0)),
        size: gpui::size(gpui::px(100.0), gpui::px(50.0)),
      },
      content_size: gpui::size(gpui::px(80.0), gpui::px(30.0)),
    };

    assert_eq!(
      box_model(&state, &style),
      BoxModel {
        element_size: gpui::size(gpui::px(100.0), gpui::px(50.0)).to_string(),
        content_size: gpui::size(gpui::px(80.0), gpui::px(30.0)).to_string(),
        content_size_compact: "80 x 30".into(),
        margin: EdgeValues {
          top: "1px".into(),
          right: "2px".into(),
          bottom: "0px".into(),
          left: "0px".into(),
        },
        border: EdgeValues {
          top: "7px".into(),
          right: "0px".into(),
          bottom: "0px".into(),
          left: "0px".into(),
        },
        padding: EdgeValues {
          top: "3px".into(),
          right: "4px".into(),
          bottom: "5px".into(),
          left: "6px".into(),
        },
      }
    );
  }

  #[test]
  fn compact_sizes_omit_pixel_units() {
    assert_eq!(
      compact_size(gpui::size(gpui::px(80.0), gpui::px(30.5))),
      "80 x 30.5"
    );
  }

  #[test]
  fn copy_feedback_ignores_stale_timeouts() {
    let mut feedback = CopyFeedback::default();
    let first = feedback.mark_copied(CopyTarget::Source, "src/lib.rs:1:1".into());
    let second = feedback.mark_copied(CopyTarget::GlobalId, "global-id".into());

    assert!(!feedback.clear(first));
    assert!(feedback.is_copied(CopyTarget::GlobalId, "global-id"));
    assert!(feedback.clear(second));
    assert!(!feedback.is_copied(CopyTarget::GlobalId, "global-id"));
  }

  #[test]
  fn clipboard_items_preserve_the_full_value() {
    let value = "view-123.long-global-id";
    assert_eq!(
      text_clipboard_item(value.into()).text(),
      Some(value.to_owned())
    );
  }

  #[test]
  fn shadows_are_formatted_like_css() {
    let mut properties = Vec::new();
    push_shadows(
      &mut properties,
      "Box shadow",
      Some(&vec![
        gpui::BoxShadow {
          color: gpui::hsla(0.0, 0.0, 0.0, 0.1),
          offset: gpui::point(gpui::px(0.0), gpui::px(4.0)),
          blur_radius: gpui::px(6.0),
          spread_radius: gpui::px(-1.0),
          inset: false,
        },
        gpui::BoxShadow {
          color: gpui::white(),
          offset: gpui::point(gpui::px(1.0), gpui::px(2.0)),
          blur_radius: gpui::px(3.0),
          spread_radius: gpui::px(0.0),
          inset: true,
        },
      ]),
    );
    assert_eq!(
      properties[0].value,
      "0px 4px 6px -1px #0000001a, inset 1px 2px 3px 0px #ffffff"
    );
  }

  #[test]
  fn empty_shadow_lists_are_skipped() {
    let mut properties = Vec::new();
    push_shadows(&mut properties, "Box shadow", Some(&Vec::new()));
    assert!(properties.is_empty());
  }

  #[test]
  fn solid_fills_show_their_color() {
    assert_eq!(format_fill(&gpui::Fill::from(gpui::white())), "#ffffff");
  }

  #[test]
  fn underlines_include_color_and_waviness() {
    let mut properties = Vec::new();
    push_underline(
      &mut properties,
      Some(&gpui::UnderlineStyle {
        thickness: gpui::px(1.0),
        color: None,
        wavy: false,
      }),
    );
    push_underline(
      &mut properties,
      Some(&gpui::UnderlineStyle {
        thickness: gpui::px(2.0),
        color: Some(gpui::white()),
        wavy: true,
      }),
    );
    assert_eq!(properties[0].value, "1px");
    assert_eq!(properties[1].value, "wavy 2px #ffffff");
  }

  #[test]
  fn strikethroughs_include_color() {
    let mut properties = Vec::new();
    push_strikethrough(
      &mut properties,
      Some(&gpui::StrikethroughStyle {
        thickness: gpui::px(1.0),
        color: Some(gpui::white()),
      }),
    );
    assert_eq!(properties[0].value, "1px #ffffff");
  }

  #[test]
  fn text_overflow_names_the_truncated_side() {
    let mut properties = Vec::new();
    push_text_overflow(
      &mut properties,
      Some(&gpui::TextOverflow::TruncateMiddle("…".into())),
    );
    assert_eq!(properties[0].value, "Truncate middle …");
  }

  #[test]
  fn font_values_are_not_debug_quoted() {
    let mut properties = Vec::new();
    push_text(&mut properties, "Font family", Some(&"monospace".into()));
    push_font_fallbacks(
      &mut properties,
      Some(&gpui::FontFallbacks::from_fonts(vec![
        "Zed Mono".into(),
        "Menlo".into(),
      ])),
    );
    push_font_features(
      &mut properties,
      Some(&gpui::FontFeatures(std::sync::Arc::new(vec![(
        "calt".into(),
        0,
      )]))),
    );
    assert_eq!(properties[0].value, "monospace");
    assert_eq!(properties[1].value, "Zed Mono, Menlo");
    assert_eq!(properties[2].value, "calt 0");
  }

  #[test]
  fn empty_font_lists_are_skipped() {
    let mut properties = Vec::new();
    push_font_fallbacks(
      &mut properties,
      Some(&gpui::FontFallbacks::from_fonts(vec![])),
    );
    push_font_features(
      &mut properties,
      Some(&gpui::FontFeatures(std::sync::Arc::new(Vec::new()))),
    );
    assert!(properties.is_empty());
  }

  #[test]
  fn colors_are_formatted_as_hex() {
    assert_eq!(format_color(gpui::white()), "#ffffff");
    assert_eq!(format_color(gpui::hsla(0.0, 1.0, 0.5, 0.5)), "#ff000080");
  }

  #[test]
  fn long_values_are_truncated_in_the_middle() {
    assert_eq!(truncate_middle("short", 10), "short");
    assert_eq!(truncate_middle("abcdefghijkl", 7), "abc…jkl");
    assert_eq!(truncate_middle("abc", 1), "…");
  }

  #[test]
  fn svg_icons_use_the_configured_color() {
    assert_eq!(
      recolor_svg(b"<svg stroke=\"currentColor\" />", 0x12abef),
      b"<svg stroke=\"#12abef\" />"
    );
  }

  #[test]
  fn source_paths_are_compact() {
    assert_eq!(
      compact_source_path("/workspace/app/src/views/card.rs"),
      "src/views/card.rs"
    );
    assert_eq!(compact_source_path("main.rs"), "main.rs");
  }
}
