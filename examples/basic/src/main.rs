use gpui::{
  App, AppContext, Bounds, Context, Window, WindowBounds, WindowOptions, div, prelude::*, px, rgb,
  size,
};

const NAV_ITEMS: [&str; 3] = ["Dashboard", "Projects", "Settings"];
const TASKS: [(&str, &str); 4] = [
  ("Design the picker overlay", "Done"),
  ("Wire up the style panel", "Done"),
  ("Polish the layout section", "In progress"),
  ("Write the README screenshot", "Todo"),
];

struct Demo;

impl Render for Demo {
  fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    div()
      .id("app-root")
      .size_full()
      .flex()
      .bg(rgb(0x101318))
      .text_color(rgb(0xe6e9ef))
      .child(sidebar())
      .child(content())
  }
}

fn sidebar() -> impl IntoElement {
  div()
    .id("sidebar")
    .w(px(200.0))
    .h_full()
    .flex_shrink_0()
    .p_4()
    .flex()
    .flex_col()
    .gap_1()
    .bg(rgb(0x14171e))
    .border_r_1()
    .border_color(rgb(0x2a2f3a))
    .child(
      div()
        .pb_3()
        .text_sm()
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .child("GPUI DevTools"),
    )
    .children(NAV_ITEMS.iter().enumerate().map(|(index, label)| {
      div()
        .id(("nav-item", index))
        .px_2()
        .py_1()
        .rounded_md()
        .text_sm()
        .text_color(rgb(0x9299a8))
        .when(index == 0, |item| {
          item.bg(rgb(0x1b2029)).text_color(rgb(0xe6e9ef))
        })
        .child(*label)
    }))
}

fn content() -> impl IntoElement {
  div()
    .id("content")
    .flex_1()
    .h_full()
    .p_6()
    .flex()
    .flex_col()
    .gap_4()
    .child(
      div()
        .id("demo-card")
        .p_6()
        .flex()
        .flex_col()
        .gap_3()
        .rounded_lg()
        .bg(rgb(0x1b2029))
        .border_1()
        .border_color(rgb(0x343b48))
        .child(div().text_xl().child("Pick an element"))
        .child("Toggle the inspector, click Pick, then select any element on this page."),
    )
    .child(task_list())
}

fn task_list() -> impl IntoElement {
  div()
    .id("task-list")
    .flex()
    .flex_col()
    .gap_2()
    .rounded_lg()
    .border_1()
    .border_color(rgb(0x343b48))
    .child(
      div()
        .px_4()
        .py_2()
        .text_sm()
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .border_b_1()
        .border_color(rgb(0x343b48))
        .child("Recent tasks"),
    )
    .children(TASKS.iter().enumerate().map(|(index, (label, status))| {
      div()
        .id(("task-row", index))
        .px_4()
        .py_2()
        .flex()
        .items_center()
        .justify_between()
        .when(index + 1 < TASKS.len(), |row| {
          row.border_b_1().border_color(rgb(0x232833))
        })
        .child(*label)
        .child(div().text_sm().text_color(rgb(0x9299a8)).child(*status))
    }))
}

fn main() {
  gpui_platform::application().run(|cx: &mut App| {
    gpui_devtools::init(cx);

    let bounds = Bounds::centered(None, size(px(960.0), px(640.0)), cx);
    cx.open_window(
      WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        ..Default::default()
      },
      |_window, cx| cx.new(|_| Demo),
    )
    .unwrap();
    cx.activate(true);
  });
}
