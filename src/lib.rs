//! Developer tools for inspecting and debugging [GPUI](https://gpui.rs) applications.
//!
//! The inspector renders next to the application window and shows the picked element's source
//! location, GPUI element ID, bounds, content size, temporary edits and `Div` style refinements.
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

use std::{cell::RefCell, rc::Rc, sync::Arc};

use gpui::{
  App, Context, Div, DivInspectorState, Inspector, InspectorElementId, IntoElement, KeyBinding,
  Window, actions, div, img, prelude::*, rgb,
};

const DEFAULT_MACOS_KEY_BINDING: &str = "cmd-alt-i";
const DEFAULT_OTHER_KEY_BINDING: &str = "ctrl-alt-i";
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

  let style_edits = Rc::new(RefCell::new(StyleEditState::default()));
  let div_config = config.clone();
  let div_style_edits = Rc::clone(&style_edits);
  cx.register_inspector_element(move |_window, _cx| {
    let div_config = div_config.clone();
    let div_style_edits = Rc::clone(&div_style_edits);
    move |id, state: &DivInspectorState, _window: &mut Window, _cx: &mut App| {
      render_div_state(id, state, &div_style_edits, &div_config)
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
  if is_picking {
    window.request_animation_frame();
  }
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
    .occlude()
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

fn render_div_state(
  id: InspectorElementId,
  state: &DivInspectorState,
  style_edits: &Rc<RefCell<StyleEditState>>,
  config: &Config,
) -> Div {
  style_edits
    .borrow_mut()
    .select(id.clone(), &state.base_style, state.bounds.size);

  div()
    .flex()
    .flex_col()
    .gap_3()
    .child(
      section("Box Model", config)
        .child(render_box_model(state, &state.base_style, config))
        .child(property("Origin", state.bounds.origin.to_string(), config)),
    )
    .child(render_style_edits(id, state, style_edits, config))
    .child(render_styles(&state.base_style, config))
}

mod box_model;
mod copy;
mod styles;
mod temporary_edits;
use box_model::render_box_model;
use copy::{CopyFeedback, CopyTarget, CopyableProperty, copyable_property};
use styles::render_styles;
use temporary_edits::{StyleEditState, render_style_edits};

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
  use crate::box_model::{BoxModel, EdgeValues, box_model, compact_size};
  use crate::copy::{CopyFeedback, CopyTarget, text_clipboard_item};
  use crate::styles::{
    StyleGroup, StyleProperty, format_color, format_fill, push_compact_sides, push_font_fallbacks,
    push_font_features, push_shadows, push_strikethrough, push_text, push_text_overflow,
    push_underline, style_groups,
  };
  use crate::temporary_edits::{
    StyleEdit, StyleEditState, active_style_edits, apply_style_edit, is_style_edit_active,
    restore_style_edit, style_edit_diff, style_edit_diff_label, style_rust_snippet, style_summary,
    style_summary_with_overrides,
  };
  use gpui::StyleRefinement;

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
  fn temporary_style_edits_change_refinements() {
    let mut style = StyleRefinement::default();
    let current_size = gpui::size(gpui::px(100.0), gpui::px(40.0));

    apply_style_edit(&mut style, StyleEdit::Wider, current_size);
    assert_eq!(style.size.width, Some(gpui::px(140.0).into()));

    apply_style_edit(&mut style, StyleEdit::Taller, current_size);
    assert_eq!(style.size.height, Some(gpui::px(80.0).into()));

    apply_style_edit(&mut style, StyleEdit::Padding, current_size);
    assert_eq!(style.padding.top, Some(gpui::px(12.0).into()));
    assert_eq!(style.padding.right, Some(gpui::px(12.0).into()));

    apply_style_edit(&mut style, StyleEdit::Margin, current_size);
    assert_eq!(style.margin.left, Some(gpui::px(8.0).into()));

    apply_style_edit(&mut style, StyleEdit::Border, current_size);
    assert_eq!(style.border_widths.top, Some(gpui::px(2.0).into()));
    assert_eq!(style.border_color, Some(rgb(0x61afef).into()));

    apply_style_edit(&mut style, StyleEdit::HalfOpacity, current_size);
    assert_eq!(style.opacity, Some(0.5));

    apply_style_edit(&mut style, StyleEdit::Hide, current_size);
    assert_eq!(style.visibility, Some(gpui::Visibility::Hidden));
  }

  #[test]
  fn style_edit_active_state_and_restore_are_per_property() {
    let original = StyleRefinement::default();
    let current_size = gpui::size(gpui::px(100.0), gpui::px(40.0));
    let mut style = original.clone();

    apply_style_edit(&mut style, StyleEdit::Hide, current_size);
    assert!(is_style_edit_active(&style, StyleEdit::Hide, current_size));
    assert!(active_style_edits(&style, current_size).contains(&StyleEdit::Hide));

    restore_style_edit(&mut style, StyleEdit::Hide, &original);
    assert!(!is_style_edit_active(&style, StyleEdit::Hide, current_size));
    assert!(!active_style_edits(&style, current_size).contains(&StyleEdit::Hide));
    assert_eq!(style.visibility, original.visibility);
  }

  #[test]
  fn active_override_labels_show_original_and_current_values() {
    let original = StyleRefinement::default();
    let current_size = gpui::size(gpui::px(100.0), gpui::px(40.0));
    let mut current = original.clone();

    apply_style_edit(&mut current, StyleEdit::Hide, current_size);
    assert_eq!(
      style_edit_diff_label(StyleEdit::Hide, &original, &current, current_size),
      "Visibility: auto -> Hidden x"
    );
    assert_eq!(
      style_edit_diff(StyleEdit::Hide, &original, &current, current_size),
      "Visibility: auto -> Hidden"
    );

    apply_style_edit(&mut current, StyleEdit::Padding, current_size);
    assert_eq!(
      style_edit_diff_label(StyleEdit::Padding, &original, &current, current_size),
      "Padding: auto -> px(12.0) x"
    );
  }

  #[test]
  fn style_summary_with_overrides_includes_active_diffs() {
    let id = InspectorElementId {
      path: std::rc::Rc::new(gpui::InspectorElementPath {
        global_id: gpui::GlobalElementId::default(),
        source_location: std::panic::Location::caller(),
      }),
      instance_id: 0,
    };
    let original = StyleRefinement::default();
    let current_size = gpui::size(gpui::px(100.0), gpui::px(40.0));
    let mut current = original.clone();
    apply_style_edit(&mut current, StyleEdit::Hide, current_size);

    let mut edits = StyleEditState::default();
    edits.select(id.clone(), &original, current_size);
    let state = DivInspectorState {
      base_style: Box::new(current),
      bounds: gpui::Bounds {
        origin: gpui::Point::default(),
        size: current_size,
      },
      content_size: current_size,
    };
    let summary = style_summary_with_overrides(&id, &state, &Rc::new(RefCell::new(edits)));

    assert!(summary.contains("Active overrides:"));
    assert!(summary.contains("Visibility: auto -> Hidden"));
    assert!(summary.contains("Full style:"));
  }

  #[test]
  fn style_exports_include_summary_and_rust_snippet() {
    let mut style = StyleRefinement::default();
    let current_size = gpui::size(gpui::px(100.0), gpui::px(40.0));
    apply_style_edit(&mut style, StyleEdit::Wider, current_size);
    apply_style_edit(&mut style, StyleEdit::Padding, current_size);
    apply_style_edit(&mut style, StyleEdit::AccentBackground, current_size);

    assert!(style_summary(&style).contains("Padding: 12px"));
    let rust = style_rust_snippet(&style);
    assert!(rust.contains(".w(px(140.0))"));
    assert!(rust.contains(".p(px(12.0))"));
    assert!(rust.contains(".bg(rgb(0x1f4f73))"));
  }

  #[test]
  fn style_rust_snippet_includes_non_uniform_sides_and_border_color() {
    let mut style = StyleRefinement::default();
    style.padding.top = Some(gpui::px(4.0).into());
    style.padding.right = Some(gpui::px(8.0).into());
    style.margin.top = Some(gpui::px(2.0).into());
    style.margin.bottom = Some(gpui::px(2.0).into());
    style.border_widths.top = Some(gpui::px(1.0).into());
    style.border_widths.left = Some(gpui::px(3.0).into());
    style.border_widths.right = Some(gpui::px(3.0).into());
    style.border_color = Some(rgb(0x61afef).into());

    let rust = style_rust_snippet(&style);
    assert!(rust.contains(".pt(px(4.0))"));
    assert!(rust.contains(".pr(px(8.0))"));
    assert!(rust.contains(".my(px(2.0))"));
    assert!(rust.contains(".border_x(px(3.0))"));
    assert!(rust.contains(".border_t(px(1.0))"));
    assert!(rust.contains(".border_color(rgb(0x61afef))"));
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
