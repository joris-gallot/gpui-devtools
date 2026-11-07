use gpui::{
  App, AppContext, Bounds, Context, FocusHandle, KeyBinding, Window, WindowBounds, WindowOptions,
  actions, div, prelude::*, px, rgb, size,
};

actions!(demo, [SelectNext, SelectPrevious, SelectFirst]);

const NAV_ITEMS: [&str; 3] = ["Dashboard", "Projects", "Settings"];
const TASKS: [(&str, &str); 4] = [
  ("Design the picker overlay", "Done"),
  ("Wire up the style panel", "Done"),
  ("Polish the layout section", "In progress"),
  ("Write the README screenshot", "Todo"),
];

struct Demo {
  task_list_focus: FocusHandle,
  selected_task: usize,
}

impl Render for Demo {
  fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    div()
      .id("app-root")
      .key_context("Demo")
      .size_full()
      .overflow_hidden()
      .flex()
      .bg(rgb(0x101318))
      .text_color(rgb(0xe6e9ef))
      .child(sidebar())
      .child(content(self, window, cx))
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

fn content(demo: &Demo, window: &Window, cx: &mut Context<Demo>) -> impl IntoElement {
  div()
    .id("content")
    .w_0()
    .flex_1()
    .h_full()
    .overflow_hidden()
    .p_6()
    .flex()
    .flex_col()
    .gap_4()
    .child(
      div()
        .id("demo-card")
        .w_full()
        .p_6()
        .flex()
        .flex_col()
        .gap_3()
        .rounded_lg()
        .bg(rgb(0x1b2029))
        .border_1()
        .border_color(rgb(0x343b48))
        .child(div().text_xl().child("Pick an element"))
        .child("Toggle the inspector, click Pick, then select any element on this page.")
        .child(
          div()
            .text_sm()
            .text_color(rgb(0x9299a8))
            .child("Click the task list, then use j, k or g g and open the Keyboard tab."),
        ),
    )
    .child(task_list(demo, window, cx))
}

fn task_list(demo: &Demo, window: &Window, cx: &mut Context<Demo>) -> impl IntoElement {
  let focused = demo.task_list_focus.is_focused(window);

  div()
    .id("task-list")
    .key_context("TaskList")
    .track_focus(&demo.task_list_focus)
    .on_click(cx.listener(|demo, _, window, cx| {
      window.focus(&demo.task_list_focus, cx);
    }))
    .on_action(cx.listener(|demo, _: &SelectNext, _, cx| {
      demo.selected_task = (demo.selected_task + 1).min(TASKS.len() - 1);
      cx.notify();
    }))
    .on_action(cx.listener(|demo, _: &SelectPrevious, _, cx| {
      demo.selected_task = demo.selected_task.saturating_sub(1);
      cx.notify();
    }))
    .on_action(cx.listener(|demo, _: &SelectFirst, _, cx| {
      demo.selected_task = 0;
      cx.notify();
    }))
    .w_full()
    .overflow_hidden()
    .flex()
    .flex_col()
    .gap_2()
    .rounded_lg()
    .border_1()
    .border_color(if focused {
      rgb(0x61afef)
    } else {
      rgb(0x343b48)
    })
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
        .when(index == demo.selected_task, |row| row.bg(rgb(0x1b2029)))
        .when(index + 1 < TASKS.len(), |row| {
          row.border_b_1().border_color(rgb(0x232833))
        })
        .child(div().w_0().flex_1().truncate().child(*label))
        .child(
          div()
            .flex_shrink_0()
            .pl_3()
            .text_sm()
            .text_color(rgb(0x9299a8))
            .child(*status),
        )
    }))
}

fn main() {
  gpui_platform::application().run(|cx: &mut App| {
    cx.bind_keys([
      KeyBinding::new("j", SelectNext, Some("TaskList")),
      KeyBinding::new("down", SelectNext, Some("TaskList")),
      KeyBinding::new("k", SelectPrevious, Some("TaskList")),
      KeyBinding::new("up", SelectPrevious, Some("TaskList")),
      KeyBinding::new("g g", SelectFirst, Some("TaskList")),
    ]);
    gpui_devtools::init(cx);

    let bounds = Bounds::centered(None, size(px(960.0), px(640.0)), cx);
    cx.open_window(
      WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        ..Default::default()
      },
      |_window, cx| {
        cx.new(|cx| Demo {
          task_list_focus: cx.focus_handle(),
          selected_task: 0,
        })
      },
    )
    .unwrap();
    cx.activate(true);
  });
}
