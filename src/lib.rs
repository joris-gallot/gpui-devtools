use std::{cell::RefCell, rc::Rc, time::Duration};

use gpui::{
  App, ClipboardItem, Context, Div, DivInspectorState, Inspector, InspectorElementId, IntoElement,
  KeyBinding, StyleRefinement, Window, actions, div, prelude::*, rgb,
};

const DEFAULT_MACOS_KEY_BINDING: &str = "cmd-alt-i";
const DEFAULT_OTHER_KEY_BINDING: &str = "ctrl-alt-i";
const COPY_FEEDBACK_DURATION: Duration = Duration::from_millis(1500);

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

  let copy_feedback = Rc::new(RefCell::new(CopyFeedback::default()));
  cx.set_inspector_renderer(Box::new(move |inspector, window, cx| {
    render_inspector(inspector, window, cx, &copy_feedback, &config).into_any_element()
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
            .h(gpui::px(28.0))
            .px_2()
            .flex()
            .items_center()
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
            .text_xs()
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
  push_debug(&mut appearance, "Box shadow", style.box_shadow.as_ref());
  push_debug(&mut appearance, "Cursor", style.mouse_cursor.as_ref());
  push_debug(&mut appearance, "Opacity", style.opacity.as_ref());
  push_group(&mut groups, "Appearance", appearance);

  if let Some(text) = style.text.explicit_refinement() {
    let mut typography = Vec::new();
    push_color(&mut typography, "Color", text.color.as_ref());
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
    push_color(
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
      value: "Color".into(),
      swatch: Some(fill.clone()),
    });
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
  fn source_paths_are_compact() {
    assert_eq!(
      compact_source_path("/workspace/app/src/views/card.rs"),
      "src/views/card.rs"
    );
    assert_eq!(compact_source_path("main.rs"), "main.rs");
  }
}
