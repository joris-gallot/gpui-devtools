use gpui::{
  App, AppContext, Application, Bounds, Context, Window, WindowBounds, WindowOptions, div,
  prelude::*, px, rgb, size,
};

struct Demo;

impl Render for Demo {
  fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    div()
      .size_full()
      .flex()
      .items_center()
      .justify_center()
      .bg(rgb(0x101318))
      .text_color(rgb(0xe6e9ef))
      .child(
        div()
          .id("demo-card")
          .w(px(360.0))
          .p_6()
          .flex()
          .flex_col()
          .gap_3()
          .rounded_lg()
          .bg(rgb(0x1b2029))
          .border_1()
          .border_color(rgb(0x343b48))
          .child(div().text_xl().child("GPUI DevTools"))
          .child("Toggle the inspector, click Pick, then select this card."),
      )
  }
}

fn main() {
  Application::new().run(|cx: &mut App| {
    gpui_devtools::init(cx);

    let bounds = Bounds::centered(None, size(px(900.0), px(600.0)), cx);
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
