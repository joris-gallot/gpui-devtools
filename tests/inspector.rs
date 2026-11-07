use std::time::Duration;

use gpui::{
  AnyElement, Context, FocusHandle, KeyBinding, Modifiers, TestAppContext, VisualTestContext,
  Window, div, prelude::*, px,
};

gpui::actions!(fixture, [Run, JumpToTop]);

struct Fixture {
  target: fn() -> (AnyElement, u32),
}

impl Render for Fixture {
  fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    div().size_full().child((self.target)().0)
  }
}

fn target() -> (AnyElement, u32) {
  let (element, line) = (div(), line!());
  (
    element
      .id("target")
      .debug_selector(|| "target".into())
      .size(px(100.0))
      .p_2()
      .into_any_element(),
    line,
  )
}

fn open_fixture(cx: &mut TestAppContext) -> &mut VisualTestContext {
  let (_, cx) = cx.add_window_view(|_, _| Fixture { target });
  cx.update(|window, _| window.activate_window());
  cx.run_until_parked();
  cx
}

fn default_shortcut() -> &'static str {
  if cfg!(target_os = "macos") {
    "cmd-alt-i"
  } else {
    "ctrl-alt-i"
  }
}

fn is_picking(cx: &mut VisualTestContext) -> bool {
  cx.update(|window, cx| window.is_inspector_picking(cx))
}

fn click_selector(cx: &mut VisualTestContext, selector: &'static str) {
  let bounds = cx
    .debug_bounds(selector)
    .unwrap_or_else(|| panic!("`{selector}` was not rendered"));
  cx.simulate_mouse_move(bounds.center(), None, Modifiers::none());
  cx.simulate_click(bounds.center(), Modifiers::none());
  cx.run_until_parked();
}

fn open_inspector(cx: &mut VisualTestContext) {
  cx.dispatch_action(gpui_devtools::ToggleInspector);
  cx.run_until_parked();
  assert!(is_picking(cx), "inspector should open in picking mode");
}

#[gpui::test]
fn toggle_action_opens_and_closes_the_inspector(cx: &mut TestAppContext) {
  cx.update(gpui_devtools::init);
  let cx = open_fixture(cx);

  open_inspector(cx);

  cx.dispatch_action(gpui_devtools::ToggleInspector);
  cx.run_until_parked();
  assert!(!is_picking(cx));
  assert!(cx.debug_bounds("gpui-devtools-pick").is_none());
}

#[gpui::test]
fn default_shortcut_toggles_the_inspector(cx: &mut TestAppContext) {
  cx.update(gpui_devtools::init);
  let cx = open_fixture(cx);

  cx.simulate_keystrokes(default_shortcut());
  cx.run_until_parked();
  assert!(is_picking(cx));
}

#[gpui::test]
fn disabled_key_binding_is_not_registered(cx: &mut TestAppContext) {
  cx.update(|cx| gpui_devtools::init_with(gpui_devtools::Config::default().key_binding(None), cx));
  let cx = open_fixture(cx);

  cx.simulate_keystrokes(default_shortcut());
  cx.run_until_parked();
  assert!(!is_picking(cx));
}

#[gpui::test]
fn close_button_closes_the_inspector(cx: &mut TestAppContext) {
  cx.update(gpui_devtools::init);
  let cx = open_fixture(cx);
  open_inspector(cx);

  click_selector(cx, "target");
  click_selector(cx, "gpui-devtools-close");
  assert!(!is_picking(cx));
  assert!(cx.debug_bounds("gpui-devtools-close").is_none());
}

#[gpui::test]
fn picking_an_element_selects_it(cx: &mut TestAppContext) {
  cx.update(gpui_devtools::init);
  let cx = open_fixture(cx);
  open_inspector(cx);

  click_selector(cx, "target");
  assert!(!is_picking(cx), "clicking an element should end picking");

  click_selector(cx, "gpui-devtools-pick");
  assert!(is_picking(cx), "the pick button should restart picking");
}

#[gpui::test]
fn temporary_style_edits_update_and_reset_the_selected_element(cx: &mut TestAppContext) {
  cx.update(gpui_devtools::init);
  let cx = open_fixture(cx);
  open_inspector(cx);

  click_selector(cx, "target");
  let initial = cx
    .debug_bounds("target")
    .expect("target should be rendered");

  click_selector(cx, "gpui-devtools-style-wider");
  let widened = cx
    .debug_bounds("target")
    .expect("target should be rendered");
  assert_eq!(widened.size.width, initial.size.width + px(40.0));

  click_selector(cx, "gpui-devtools-style-reset");
  let reset = cx
    .debug_bounds("target")
    .expect("target should be rendered");
  assert_eq!(reset.size.width, initial.size.width);
}

#[gpui::test]
fn temporary_style_edits_toggle_their_override(cx: &mut TestAppContext) {
  cx.update(gpui_devtools::init);
  let cx = open_fixture(cx);
  open_inspector(cx);

  click_selector(cx, "target");
  click_selector(cx, "gpui-devtools-style-hide");
  click_selector(cx, "gpui-devtools-copy-style-summary");
  let copied = cx
    .read_from_clipboard()
    .and_then(|item| item.text())
    .expect("style summary should be copied");
  assert!(copied.contains("Visibility: Hidden"));

  click_selector(cx, "gpui-devtools-remove-style-hide");
  click_selector(cx, "gpui-devtools-copy-style-summary");
  let copied = cx
    .read_from_clipboard()
    .and_then(|item| item.text())
    .expect("style summary should be copied");
  assert!(!copied.contains("Visibility: Hidden"));
}

#[gpui::test]
fn style_export_buttons_copy_the_selected_element_styles(cx: &mut TestAppContext) {
  cx.update(gpui_devtools::init);
  let cx = open_fixture(cx);
  open_inspector(cx);

  click_selector(cx, "target");
  click_selector(cx, "gpui-devtools-style-padding");
  click_selector(cx, "gpui-devtools-copy-style-summary");

  let copied = cx
    .read_from_clipboard()
    .and_then(|item| item.text())
    .expect("style summary should be copied");
  assert!(copied.contains("Active overrides:"));
  assert!(copied.contains("Padding:"));
  assert!(copied.contains("-> px(12.0)"));
  assert!(copied.contains("Full style:"));
  assert!(copied.contains("Spacing:"));
  assert!(copied.contains("Padding: 12px"));

  click_selector(cx, "gpui-devtools-copy-style-rust");
  let copied = cx
    .read_from_clipboard()
    .and_then(|item| item.text())
    .expect("style Rust snippet should be copied");
  assert!(copied.contains("div()"));
  assert!(copied.contains(".p(px(12.0))"));
}

#[gpui::test]
fn copy_source_writes_the_selected_element_location(cx: &mut TestAppContext) {
  cx.update(gpui_devtools::init);
  let cx = open_fixture(cx);
  open_inspector(cx);

  click_selector(cx, "target");
  click_selector(cx, "gpui-devtools-copy-source");

  let copied = cx
    .read_from_clipboard()
    .and_then(|item| item.text())
    .expect("source location should be copied");
  let expected_prefix = format!("inspector.rs:{}:", target().1);
  assert!(
    copied.starts_with(&expected_prefix),
    "expected `{copied}` to start with `{expected_prefix}`"
  );

  cx.executor().advance_clock(Duration::from_secs(2));
  cx.run_until_parked();
}

#[gpui::test]
fn copy_global_id_writes_the_full_id(cx: &mut TestAppContext) {
  cx.update(gpui_devtools::init);
  let cx = open_fixture(cx);
  open_inspector(cx);

  click_selector(cx, "target");
  click_selector(cx, "gpui-devtools-copy-global-id");

  let copied = cx
    .read_from_clipboard()
    .and_then(|item| item.text())
    .expect("global ID should be copied");
  assert!(
    copied.contains("target"),
    "expected `{copied}` to contain the element ID"
  );
}

struct KeyboardFixture {
  focus: FocusHandle,
}

impl Render for KeyboardFixture {
  fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    div()
      .size_full()
      .key_context("Fixture")
      .track_focus(&self.focus)
      .on_action(|_: &Run, _, _| {})
      .on_action(|_: &JumpToTop, _, _| {})
      .child(
        div()
          .id("target")
          .debug_selector(|| "target".into())
          .size(px(100.0)),
      )
  }
}

fn open_keyboard_tab(cx: &mut TestAppContext) -> &mut VisualTestContext {
  cx.update(|cx| {
    gpui_devtools::init(cx);
    cx.bind_keys([
      KeyBinding::new("ctrl-k", Run, Some("Fixture")),
      KeyBinding::new("g g", JumpToTop, Some("Fixture")),
    ]);
  });
  let (view, cx) = cx.add_window_view(|_, cx| KeyboardFixture {
    focus: cx.focus_handle(),
  });
  cx.update(|window, cx| {
    window.activate_window();
    let focus = view.read(cx).focus.clone();
    window.focus(&focus, cx);
  });
  cx.run_until_parked();

  open_inspector(cx);
  click_selector(cx, "target");
  click_selector(cx, "gpui-devtools-tab-keyboard");
  cx
}

fn keyboard_report(cx: &mut VisualTestContext) -> String {
  click_selector(cx, "gpui-devtools-copy-keyboard-report");
  cx.read_from_clipboard()
    .and_then(|item| item.text())
    .expect("keyboard report should be copied")
}

#[gpui::test]
fn keyboard_tab_reports_focus_contexts_and_bindings(cx: &mut TestAppContext) {
  let cx = open_keyboard_tab(cx);

  let report = keyboard_report(cx);
  assert!(report.contains("Focus: focused element"), "{report}");
  assert!(
    report.contains("Key contexts (root to focus):\n  Fixture"),
    "{report}"
  );
  assert!(report.contains("  fixture::Run (ctrl-k)"), "{report}");
  assert!(report.contains("Pending input: none"), "{report}");
}

#[gpui::test]
fn keyboard_tab_records_keystrokes_and_their_actions(cx: &mut TestAppContext) {
  let cx = open_keyboard_tab(cx);

  cx.simulate_keystrokes("ctrl-k x");
  cx.run_until_parked();
  let report = keyboard_report(cx);
  assert!(
    report.contains("  x -> no action [Fixture]\n  ctrl-k -> fixture::Run [Fixture]"),
    "{report}"
  );

  click_selector(cx, "gpui-devtools-clear-keystrokes");
  let report = keyboard_report(cx);
  assert!(
    report.contains("Recent keystrokes (newest first):\n  none"),
    "{report}"
  );
}

#[gpui::test]
fn keyboard_tab_shows_pending_multi_stroke_input(cx: &mut TestAppContext) {
  let cx = open_keyboard_tab(cx);

  cx.simulate_keystrokes("g");
  cx.run_until_parked();
  let report = keyboard_report(cx);
  assert!(report.contains("Pending input: g"), "{report}");
  assert!(report.contains("  g g -> fixture::JumpToTop"), "{report}");

  cx.simulate_keystrokes("g");
  cx.run_until_parked();
  let report = keyboard_report(cx);
  assert!(report.contains("Pending input: none"), "{report}");
  assert!(
    report.contains("  g -> fixture::JumpToTop [Fixture]"),
    "{report}"
  );
}

#[gpui::test]
fn picking_switches_back_to_the_element_tab(cx: &mut TestAppContext) {
  let cx = open_keyboard_tab(cx);
  assert!(
    cx.debug_bounds("gpui-devtools-copy-keyboard-report")
      .is_some()
  );

  click_selector(cx, "gpui-devtools-pick");
  assert!(
    cx.debug_bounds("gpui-devtools-copy-keyboard-report")
      .is_none()
  );
}
