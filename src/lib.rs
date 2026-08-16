use gpui::{
  App, Context, Div, DivInspectorState, Inspector, InspectorElementId, IntoElement, KeyBinding,
  StyleRefinement, Window, actions, div, prelude::*, rgb,
};

const DEFAULT_MACOS_KEY_BINDING: &str = "cmd-alt-i";
const DEFAULT_OTHER_KEY_BINDING: &str = "ctrl-alt-i";

actions!(gpui_devtools, [ToggleInspector]);

#[derive(Clone, Debug)]
pub struct Config {
  pub key_binding: Option<&'static str>,
  pub background: u32,
  pub panel_background: u32,
  pub border: u32,
  pub text: u32,
  pub muted_text: u32,
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
  pub fn key_binding(mut self, key_binding: Option<&'static str>) -> Self {
    self.key_binding = key_binding;
    self
  }
}

pub fn init(cx: &mut App) {
  init_with(Config::default(), cx);
}

pub fn init_with(config: Config, cx: &mut App) {
  if let Some(key_binding) = config.key_binding {
    cx.bind_keys([KeyBinding::new(key_binding, ToggleInspector, None)]);
  }

  cx.on_action(|_: &ToggleInspector, cx| toggle_active_window(cx));

  let div_config = config.clone();
  cx.register_inspector_element(move |_id, state: &DivInspectorState, _window, _cx| {
    render_div_state(state, &div_config)
  });

  cx.set_inspector_renderer(Box::new(move |inspector, window, cx| {
    render_inspector(inspector, window, cx, &config).into_any_element()
  }));
}

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
      .child(render_element_id(&id, config))
      .children(inspector_states)
  } else {
    content.child(render_empty_state(is_picking, config))
  };

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
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .child("GPUI DevTools"),
        )
        .child(
          div()
            .id("gpui-devtools-pick")
            .px_2()
            .py_1()
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
            .child(if is_picking { "Picking..." } else { "Pick" })
            .on_click(cx.listener(|inspector, _, window, _cx| {
              inspector.start_picking();
              window.refresh();
            })),
        ),
    )
    .child(content)
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

fn render_element_id(id: &InspectorElementId, config: &Config) -> Div {
  section("Selected element", config)
    .child(property("Source", source_location(id), config))
    .child(
      div()
        .flex()
        .gap_3()
        .child(property("Instance", id.instance_id.to_string(), config))
        .child(
          property(
            "Global ID",
            truncate_middle(&id.path.global_id.to_string(), 48),
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
      section("Layout", config)
        .child(render_geometry(state, config))
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
    .children(
      group
        .properties
        .into_iter()
        .map(|property| render_style_property(property, config)),
    )
}

fn render_style_property(property: StyleProperty, config: &Config) -> Div {
  div()
    .py_1()
    .flex()
    .items_start()
    .justify_between()
    .gap_3()
    .border_b_1()
    .border_color(rgb(config.border))
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
        .truncate()
        .font_family("monospace")
        .text_right()
        .child(property.value),
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
  push_debug(&mut spacing, "Margin top", style.margin.top.as_ref());
  push_debug(&mut spacing, "Margin right", style.margin.right.as_ref());
  push_debug(&mut spacing, "Margin bottom", style.margin.bottom.as_ref());
  push_debug(&mut spacing, "Margin left", style.margin.left.as_ref());
  push_debug(&mut spacing, "Padding top", style.padding.top.as_ref());
  push_debug(&mut spacing, "Padding right", style.padding.right.as_ref());
  push_debug(
    &mut spacing,
    "Padding bottom",
    style.padding.bottom.as_ref(),
  );
  push_debug(&mut spacing, "Padding left", style.padding.left.as_ref());
  push_debug(&mut spacing, "Border top", style.border_widths.top.as_ref());
  push_debug(
    &mut spacing,
    "Border right",
    style.border_widths.right.as_ref(),
  );
  push_debug(
    &mut spacing,
    "Border bottom",
    style.border_widths.bottom.as_ref(),
  );
  push_debug(
    &mut spacing,
    "Border left",
    style.border_widths.left.as_ref(),
  );
  push_group(&mut groups, "Spacing", spacing);

  let mut appearance = Vec::new();
  push_debug(&mut appearance, "Background", style.background.as_ref());
  push_debug(&mut appearance, "Border color", style.border_color.as_ref());
  push_debug(&mut appearance, "Border style", style.border_style.as_ref());
  push_debug(
    &mut appearance,
    "Radius top left",
    style.corner_radii.top_left.as_ref(),
  );
  push_debug(
    &mut appearance,
    "Radius top right",
    style.corner_radii.top_right.as_ref(),
  );
  push_debug(
    &mut appearance,
    "Radius bottom right",
    style.corner_radii.bottom_right.as_ref(),
  );
  push_debug(
    &mut appearance,
    "Radius bottom left",
    style.corner_radii.bottom_left.as_ref(),
  );
  push_debug(&mut appearance, "Box shadow", style.box_shadow.as_ref());
  push_debug(&mut appearance, "Cursor", style.mouse_cursor.as_ref());
  push_debug(&mut appearance, "Opacity", style.opacity.as_ref());
  push_group(&mut groups, "Appearance", appearance);

  if let Some(text) = style.text.explicit_refinement() {
    let mut typography = Vec::new();
    push_debug(&mut typography, "Color", text.color.as_ref());
    push_debug(&mut typography, "Font family", text.font_family.as_ref());
    push_debug(
      &mut typography,
      "Font features",
      text.font_features.as_ref(),
    );
    push_debug(
      &mut typography,
      "Font fallbacks",
      text.font_fallbacks.as_ref(),
    );
    push_debug(&mut typography, "Font size", text.font_size.as_ref());
    push_debug(&mut typography, "Line height", text.line_height.as_ref());
    push_debug(&mut typography, "Font weight", text.font_weight.as_ref());
    push_debug(&mut typography, "Font style", text.font_style.as_ref());
    push_debug(
      &mut typography,
      "Background",
      text.background_color.as_ref(),
    );
    push_debug(&mut typography, "Underline", text.underline.as_ref());
    push_debug(
      &mut typography,
      "Strikethrough",
      text.strikethrough.as_ref(),
    );
    push_debug(&mut typography, "White space", text.white_space.as_ref());
    push_debug(
      &mut typography,
      "Text overflow",
      text.text_overflow.as_ref(),
    );
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

fn push_debug<T: std::fmt::Debug>(
  properties: &mut Vec<StyleProperty>,
  label: &'static str,
  value: Option<&T>,
) {
  if let Some(value) = value {
    properties.push(StyleProperty {
      label,
      value: format!("{value:?}"),
    });
  }
}

fn push_group(groups: &mut Vec<StyleGroup>, label: &'static str, properties: Vec<StyleProperty>) {
  if !properties.is_empty() {
    groups.push(StyleGroup { label, properties });
  }
}

fn render_geometry(state: &DivInspectorState, config: &Config) -> Div {
  div()
    .p_2()
    .rounded_md()
    .border_1()
    .border_color(rgb(config.accent))
    .bg(rgb(config.background))
    .child(geometry_label(
      "Element",
      state.bounds.size.to_string(),
      config,
    ))
    .child(
      div()
        .mt_2()
        .p_3()
        .rounded_md()
        .border_1()
        .border_color(rgb(config.border))
        .bg(rgb(config.panel_background))
        .child(geometry_label(
          "Content",
          state.content_size.to_string(),
          config,
        )),
    )
}

fn geometry_label(label: &'static str, value: String, config: &Config) -> Div {
  div()
    .flex()
    .items_center()
    .justify_between()
    .text_xs()
    .child(div().text_color(rgb(config.muted_text)).child(label))
    .child(div().font_family("monospace").child(value))
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
  div()
    .overflow_hidden()
    .flex()
    .flex_col()
    .gap_1()
    .child(
      div()
        .text_xs()
        .text_color(rgb(config.muted_text))
        .child(label),
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
        }],
      }]
    );
  }

  #[test]
  fn long_values_are_truncated_in_the_middle() {
    assert_eq!(truncate_middle("short", 10), "short");
    assert_eq!(truncate_middle("abcdefghijkl", 7), "abc…jkl");
    assert_eq!(truncate_middle("abc", 1), "…");
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
