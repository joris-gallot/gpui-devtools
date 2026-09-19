use std::time::Duration;

use gpui::{
  AnyElement, Context, Modifiers, TestAppContext, VisualTestContext, Window, div, prelude::*, px,
};

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
