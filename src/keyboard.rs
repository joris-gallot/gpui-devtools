use std::{
  cell::RefCell,
  collections::{HashMap, VecDeque},
  rc::Rc,
  time::Duration,
};

use gpui::{
  App, Div, KeyBinding, KeyContext, KeystrokeEvent, OperatingSystem, Window, WindowId, div,
  prelude::*, rgb,
};

use crate::{
  Config,
  copy::text_clipboard_item,
  ui::{inspector_button, property, section},
};

pub(crate) const MAX_KEYSTROKES: usize = 20;
const PENDING_REFRESH_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct KeystrokeRecord {
  pub(crate) keystroke: String,
  pub(crate) action: Option<String>,
  pub(crate) contexts: Vec<String>,
}

#[derive(Default)]
struct WindowKeyboardLog {
  keystrokes: VecDeque<KeystrokeRecord>,
  panel_rendered: bool,
}

#[derive(Default)]
pub(crate) struct KeyboardLog {
  windows: HashMap<WindowId, WindowKeyboardLog>,
}

impl KeyboardLog {
  pub(crate) fn record(&mut self, window: WindowId, record: KeystrokeRecord) {
    let keystrokes = &mut self.windows.entry(window).or_default().keystrokes;
    keystrokes.push_front(record);
    keystrokes.truncate(MAX_KEYSTROKES);
  }

  pub(crate) fn keystrokes(&self, window: WindowId) -> Vec<KeystrokeRecord> {
    self
      .windows
      .get(&window)
      .map(|log| log.keystrokes.iter().cloned().collect())
      .unwrap_or_default()
  }

  pub(crate) fn clear(&mut self, window: WindowId) {
    if let Some(log) = self.windows.get_mut(&window) {
      log.keystrokes.clear();
    }
  }

  pub(crate) fn mark_panel_rendered(&mut self, window: WindowId) {
    self.windows.entry(window).or_default().panel_rendered = true;
  }

  /// Returns whether the keyboard panel was drawn since the last call, so closed panels stop
  /// forcing redraws after one keystroke.
  pub(crate) fn take_panel_rendered(&mut self, window: WindowId) -> bool {
    self
      .windows
      .get_mut(&window)
      .is_some_and(|log| std::mem::take(&mut log.panel_rendered))
  }
}

pub(crate) fn observe_keyboard(log: &Rc<RefCell<KeyboardLog>>, cx: &mut App) {
  let interceptor_log = Rc::clone(log);
  cx.intercept_keystrokes(move |_, window, _| {
    let window_id = window.window_handle().window_id();
    if interceptor_log.borrow_mut().take_panel_rendered(window_id) {
      window.refresh();
    }
  })
  .detach();

  let observer_log = Rc::clone(log);
  cx.observe_keystrokes(move |event, window, cx| {
    let record = keystroke_record(event, cx.platform_info().os);
    observer_log
      .borrow_mut()
      .record(window.window_handle().window_id(), record);
  })
  .detach();
}

fn keystroke_record(event: &KeystrokeEvent, os: OperatingSystem) -> KeystrokeRecord {
  KeystrokeRecord {
    keystroke: event.keystroke.unparse(os),
    action: event.action.as_ref().map(|action| action.name().to_owned()),
    contexts: event.context_stack.iter().map(format_context).collect(),
  }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ActionBinding {
  pub(crate) action: String,
  pub(crate) binding: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct KeyboardSnapshot {
  pub(crate) focused: bool,
  pub(crate) window_active: bool,
  pub(crate) contexts: Vec<String>,
  pub(crate) pending: Option<String>,
  pub(crate) next_bindings: Vec<ActionBinding>,
  pub(crate) actions: Vec<ActionBinding>,
  pub(crate) keystrokes: Vec<KeystrokeRecord>,
}

impl KeyboardSnapshot {
  fn capture(window: &Window, cx: &App, keystrokes: Vec<KeystrokeRecord>) -> Self {
    let os = cx.platform_info().os;
    let contexts = window.context_stack();
    let keymap = cx.key_bindings();
    let keymap = keymap.borrow();

    let mut actions = window
      .available_actions(cx)
      .iter()
      .map(|action| ActionBinding {
        action: action.name().to_owned(),
        binding: active_binding(keymap.bindings_for_action(action.as_ref()), &contexts)
          .map(|binding| format_binding(binding, os)),
      })
      .collect::<Vec<_>>();
    actions.sort_by(|a, b| a.action.cmp(&b.action));
    actions.dedup();

    let pending = window.pending_input_keystrokes().map(|keystrokes| {
      keystrokes
        .iter()
        .map(|keystroke| keystroke.unparse(os))
        .collect::<Vec<_>>()
        .join(" ")
    });
    let next_bindings = window
      .pending_input_keystrokes()
      .map(|keystrokes| window.possible_bindings_for_input(keystrokes))
      .unwrap_or_default()
      .iter()
      .map(|binding| ActionBinding {
        action: binding.action().name().to_owned(),
        binding: Some(format_binding(binding, os)),
      })
      .collect();

    Self {
      focused: window.focused(cx).is_some(),
      window_active: window.is_window_active(),
      contexts: contexts.iter().map(format_context).collect(),
      pending,
      next_bindings,
      actions,
      keystrokes,
    }
  }

  pub(crate) fn report(&self) -> String {
    let mut lines = vec![
      format!("Focus: {}", focus_label(self.focused)),
      format!("Window active: {}", yes_no(self.window_active)),
      "Key contexts (root to focus):".to_owned(),
    ];
    if self.contexts.is_empty() {
      lines.push("  none".to_owned());
    }
    lines.extend(self.contexts.iter().map(|context| format!("  {context}")));

    lines.push(format!(
      "Pending input: {}",
      self.pending.as_deref().unwrap_or("none")
    ));
    lines.extend(self.next_bindings.iter().map(|next| {
      format!(
        "  {} -> {}",
        next.binding.as_deref().unwrap_or(""),
        next.action
      )
    }));

    lines.push("Available actions:".to_owned());
    if self.actions.is_empty() {
      lines.push("  none".to_owned());
    }
    lines.extend(self.actions.iter().map(|action| match &action.binding {
      Some(binding) => format!("  {} ({binding})", action.action),
      None => format!("  {}", action.action),
    }));

    lines.push("Recent keystrokes (newest first):".to_owned());
    if self.keystrokes.is_empty() {
      lines.push("  none".to_owned());
    }
    lines.extend(self.keystrokes.iter().map(|record| {
      format!(
        "  {} -> {} [{}]",
        record.keystroke,
        record.action.as_deref().unwrap_or("no action"),
        record.contexts.join(" > ")
      )
    }));

    lines.join("\n")
  }
}

/// Picks the binding GPUI would favor for display: the deepest matching context wins, then the
/// most recently added binding.
pub(crate) fn active_binding<'a>(
  bindings: impl Iterator<Item = &'a KeyBinding>,
  contexts: &[KeyContext],
) -> Option<&'a KeyBinding> {
  bindings
    .filter_map(|binding| match binding.predicate() {
      None => Some((0, binding)),
      Some(predicate) => predicate.depth_of(contexts).map(|depth| (depth, binding)),
    })
    .enumerate()
    .max_by_key(|(index, (depth, _))| (*depth, *index))
    .map(|(_, (_, binding))| binding)
}

pub(crate) fn format_binding(binding: &KeyBinding, os: OperatingSystem) -> String {
  binding
    .keystrokes()
    .iter()
    .map(|keystroke| keystroke.unparse(os))
    .collect::<Vec<_>>()
    .join(" ")
}

pub(crate) fn format_context(context: &KeyContext) -> String {
  format!("{context:?}")
}

fn focus_label(focused: bool) -> &'static str {
  if focused { "focused element" } else { "none" }
}

fn yes_no(value: bool) -> &'static str {
  if value { "yes" } else { "no" }
}

pub(crate) fn render_keyboard(
  window: &mut Window,
  cx: &App,
  log: &Rc<RefCell<KeyboardLog>>,
  config: &Config,
) -> Div {
  let window_id = window.window_handle().window_id();
  let keystrokes = {
    let mut log = log.borrow_mut();
    log.mark_panel_rendered(window_id);
    log.keystrokes(window_id)
  };
  let snapshot = KeyboardSnapshot::capture(window, cx, keystrokes);
  if snapshot.pending.is_some() {
    // Pending input can expire on a timer that does not redraw the window.
    window
      .spawn(cx, async move |cx| {
        cx.background_executor()
          .timer(PENDING_REFRESH_INTERVAL)
          .await;
        let _ = cx.update(|window, _| window.refresh());
      })
      .detach();
  }

  let report = snapshot.report();
  let clear_log = Rc::clone(log);

  div()
    .flex()
    .flex_col()
    .gap_3()
    .child(
      section("Focus", config)
        .child(property(
          "Focused element",
          if snapshot.focused {
            "Yes".to_owned()
          } else {
            "None, keys dispatch from the window root".to_owned()
          },
          config,
        ))
        .child(property(
          "Window active",
          if snapshot.window_active { "Yes" } else { "No" }.to_owned(),
          config,
        )),
    )
    .child(
      section("Key contexts", config)
        .child(muted_line("Root to focus", config))
        .when(snapshot.contexts.is_empty(), |section| {
          section.child(muted_line("No key context on the focus path", config))
        })
        .children(
          snapshot
            .contexts
            .iter()
            .enumerate()
            .map(|(depth, context)| {
              mono_line(context.clone(), config).pl(gpui::px(depth as f32 * 8.0))
            }),
        ),
    )
    .when_some(snapshot.pending.clone(), |panel, pending| {
      panel.child(
        section("Pending input", config)
          .child(property("Keystrokes", pending, config))
          .children(
            snapshot
              .next_bindings
              .iter()
              .map(|next| binding_row(next, config)),
          ),
      )
    })
    .child(
      section(
        format!("Available actions ({})", snapshot.actions.len()),
        config,
      )
      .when(snapshot.actions.is_empty(), |section| {
        section.child(muted_line("No action available for the focus", config))
      })
      .children(
        snapshot
          .actions
          .iter()
          .map(|action| binding_row(action, config)),
      ),
    )
    .child(
      section("Recent keystrokes", config)
        .when(snapshot.keystrokes.is_empty(), |section| {
          section.child(muted_line(
            "Press keys in the application to record them.",
            config,
          ))
        })
        .children(
          snapshot
            .keystrokes
            .iter()
            .map(|record| keystroke_row(record, config)),
        )
        .child(
          div()
            .flex()
            .gap_2()
            .child(
              inspector_button(
                "gpui-devtools-clear-keystrokes",
                "Clear".to_owned(),
                true,
                false,
                config,
              )
              .on_click(move |_, window, _| {
                clear_log
                  .borrow_mut()
                  .clear(window.window_handle().window_id());
                window.refresh();
              }),
            )
            .child(
              inspector_button(
                "gpui-devtools-copy-keyboard-report",
                "Copy report".to_owned(),
                false,
                false,
                config,
              )
              .on_click(move |_, _, cx| {
                cx.write_to_clipboard(text_clipboard_item(report.clone()));
              }),
            ),
        ),
    )
}

fn binding_row(action: &ActionBinding, config: &Config) -> Div {
  div()
    .flex()
    .items_center()
    .justify_between()
    .gap_2()
    .text_xs()
    .font_family("monospace")
    .child(div().w_0().flex_1().truncate().child(action.action.clone()))
    .when_some(action.binding.clone(), |row, binding| {
      row.child(
        div()
          .flex_shrink_0()
          .px_1()
          .rounded_sm()
          .bg(rgb(config.background))
          .text_color(rgb(config.accent))
          .child(binding),
      )
    })
}

fn keystroke_row(record: &KeystrokeRecord, config: &Config) -> Div {
  div()
    .flex()
    .flex_col()
    .gap_1()
    .text_xs()
    .font_family("monospace")
    .child(
      div()
        .flex()
        .gap_2()
        .child(
          div()
            .flex_shrink_0()
            .text_color(rgb(config.accent))
            .child(record.keystroke.clone()),
        )
        .child(
          div()
            .w_0()
            .flex_1()
            .truncate()
            .when(record.action.is_none(), |label| {
              label.text_color(rgb(config.muted_text))
            })
            .child(
              record
                .action
                .clone()
                .unwrap_or_else(|| "no action".to_owned()),
            ),
        ),
    )
    .child(div().truncate().text_color(rgb(config.muted_text)).child(
      if record.contexts.is_empty() {
        "no key context".to_owned()
      } else {
        record.contexts.join(" > ")
      },
    ))
}

fn mono_line(text: String, config: &Config) -> Div {
  div()
    .truncate()
    .text_sm()
    .font_family("monospace")
    .text_color(rgb(config.text))
    .child(text)
}

fn muted_line(text: &'static str, config: &Config) -> Div {
  div()
    .text_xs()
    .text_color(rgb(config.muted_text))
    .child(text)
}

#[cfg(test)]
mod tests {
  use super::*;

  gpui::actions!(keyboard_tests, [First, Second]);

  fn record(keystroke: &str) -> KeystrokeRecord {
    KeystrokeRecord {
      keystroke: keystroke.to_owned(),
      action: None,
      contexts: Vec::new(),
    }
  }

  fn window_id(id: u64) -> WindowId {
    WindowId::from(id)
  }

  #[test]
  fn keystroke_log_keeps_the_newest_entries_per_window() {
    let mut log = KeyboardLog::default();
    for index in 0..MAX_KEYSTROKES + 5 {
      log.record(window_id(1), record(&index.to_string()));
    }
    log.record(window_id(2), record("other"));

    let keystrokes = log.keystrokes(window_id(1));
    assert_eq!(keystrokes.len(), MAX_KEYSTROKES);
    assert_eq!(keystrokes[0].keystroke, (MAX_KEYSTROKES + 4).to_string());
    assert_eq!(log.keystrokes(window_id(2)), vec![record("other")]);

    log.clear(window_id(1));
    assert!(log.keystrokes(window_id(1)).is_empty());
    assert_eq!(log.keystrokes(window_id(2)).len(), 1);
  }

  #[test]
  fn panel_rendered_flag_is_consumed_once() {
    let mut log = KeyboardLog::default();
    assert!(!log.take_panel_rendered(window_id(1)));

    log.mark_panel_rendered(window_id(1));
    assert!(log.take_panel_rendered(window_id(1)));
    assert!(!log.take_panel_rendered(window_id(1)));
  }

  #[test]
  fn active_binding_prefers_deeper_contexts_then_later_bindings() {
    let bindings = [
      KeyBinding::new("ctrl-a", First, None),
      KeyBinding::new("ctrl-b", First, Some("Editor")),
      KeyBinding::new("ctrl-c", First, Some("Workspace")),
      KeyBinding::new("ctrl-d", First, Some("Missing")),
    ];
    let contexts = [
      KeyContext::parse("Workspace").unwrap(),
      KeyContext::parse("Editor").unwrap(),
    ];

    let binding = active_binding(bindings.iter(), &contexts).unwrap();
    assert_eq!(format_binding(binding, OperatingSystem::Linux), "ctrl-b");

    let binding = active_binding(bindings.iter(), &contexts[..1]).unwrap();
    assert_eq!(format_binding(binding, OperatingSystem::Linux), "ctrl-c");

    let binding = active_binding(bindings.iter(), &[]).unwrap();
    assert_eq!(format_binding(binding, OperatingSystem::Linux), "ctrl-a");

    let later = [
      KeyBinding::new("ctrl-a", Second, None),
      KeyBinding::new("ctrl-z", Second, None),
    ];
    let binding = active_binding(later.iter(), &[]).unwrap();
    assert_eq!(format_binding(binding, OperatingSystem::Linux), "ctrl-z");
  }

  #[test]
  fn multi_stroke_bindings_are_space_separated() {
    let binding = KeyBinding::new("g g", First, None);
    assert_eq!(format_binding(&binding, OperatingSystem::Linux), "g g");
  }

  #[test]
  fn report_lists_every_keyboard_section() {
    let snapshot = KeyboardSnapshot {
      focused: true,
      window_active: true,
      contexts: vec!["Workspace".into(), "Editor mode=full".into()],
      pending: Some("g".into()),
      next_bindings: vec![ActionBinding {
        action: "editor::Top".into(),
        binding: Some("g g".into()),
      }],
      actions: vec![
        ActionBinding {
          action: "editor::Copy".into(),
          binding: Some("ctrl-c".into()),
        },
        ActionBinding {
          action: "editor::Noop".into(),
          binding: None,
        },
      ],
      keystrokes: vec![KeystrokeRecord {
        keystroke: "ctrl-c".into(),
        action: Some("editor::Copy".into()),
        contexts: vec!["Workspace".into(), "Editor mode=full".into()],
      }],
    };

    assert_eq!(
      snapshot.report(),
      [
        "Focus: focused element",
        "Window active: yes",
        "Key contexts (root to focus):",
        "  Workspace",
        "  Editor mode=full",
        "Pending input: g",
        "  g g -> editor::Top",
        "Available actions:",
        "  editor::Copy (ctrl-c)",
        "  editor::Noop",
        "Recent keystrokes (newest first):",
        "  ctrl-c -> editor::Copy [Workspace > Editor mode=full]",
      ]
      .join("\n")
    );
  }

  #[test]
  fn empty_report_marks_missing_values() {
    let report = KeyboardSnapshot::default().report();
    assert!(report.contains("Focus: none"));
    assert!(report.contains("Pending input: none"));
    assert!(report.contains("Recent keystrokes (newest first):\n  none"));
  }
}
