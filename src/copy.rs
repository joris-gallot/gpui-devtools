use std::{cell::RefCell, rc::Rc, time::Duration};

use gpui::{ClipboardItem, Context, Div, Inspector, IntoElement, div, prelude::*, rgb};

use crate::{Config, ui::property_with_action};

const COPY_FEEDBACK_DURATION: Duration = Duration::from_millis(1500);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CopyTarget {
  Source,
  GlobalId,
}

#[derive(Debug, Default)]
pub(crate) struct CopyFeedback {
  copied: Option<(CopyTarget, String)>,
  generation: u64,
}

impl CopyFeedback {
  pub(crate) fn is_copied(&self, target: CopyTarget, value: &str) -> bool {
    self
      .copied
      .as_ref()
      .is_some_and(|copied| copied.0 == target && copied.1 == value)
  }

  pub(crate) fn mark_copied(&mut self, target: CopyTarget, value: String) -> u64 {
    self.generation = self.generation.wrapping_add(1);
    self.copied = Some((target, value));
    self.generation
  }

  pub(crate) fn clear(&mut self, generation: u64) -> bool {
    if self.generation != generation {
      return false;
    }

    self.copied = None;
    true
  }
}

pub(crate) struct CopyableProperty {
  pub(crate) id: &'static str,
  pub(crate) label: &'static str,
  pub(crate) display_value: String,
  pub(crate) copy_value: String,
  pub(crate) target: CopyTarget,
}

pub(crate) fn copyable_property(
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

pub(crate) fn text_clipboard_item(value: String) -> ClipboardItem {
  ClipboardItem::new_string(value)
}
