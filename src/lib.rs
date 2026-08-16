use gpui::{
    App, Context, Div, DivInspectorState, Inspector, InspectorElementId, IntoElement, KeyBinding,
    Window, actions, div, prelude::*, rgb,
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
                        .bg(if inspector.is_picking() {
                            rgb(config.accent)
                        } else {
                            rgb(config.panel_background)
                        })
                        .child(if inspector.is_picking() {
                            "Picking"
                        } else {
                            "Pick"
                        })
                        .on_click(cx.listener(|inspector, _, window, _cx| {
                            inspector.start_picking();
                            window.refresh();
                        })),
                ),
        )
        .child(
            div()
                .id("gpui-devtools-content")
                .flex_1()
                .overflow_y_scroll()
                .p_3()
                .flex()
                .flex_col()
                .gap_3()
                .when_some(active_element, |panel, id| {
                    panel.child(render_element_id(&id, config))
                })
                .children(inspector.render_inspector_states(window, cx)),
        )
}

fn render_element_id(id: &InspectorElementId, config: &Config) -> Div {
    let location = source_location(id);

    section("Element", config)
        .child(property("Source", location, config))
        .child(property("Instance", id.instance_id.to_string(), config))
        .child(property("Global ID", id.path.global_id.to_string(), config))
}

fn render_div_state(state: &DivInspectorState, config: &Config) -> Div {
    section("Layout", config)
        .child(property("Origin", state.bounds.origin.to_string(), config))
        .child(property("Size", state.bounds.size.to_string(), config))
        .child(property("Content", state.content_size.to_string(), config))
        .child(property(
            "Style refinement",
            format!("{:#?}", state.base_style),
            config,
        ))
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
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .text_xs()
                .text_color(rgb(config.muted_text))
                .child(label),
        )
        .child(div().text_sm().font_family("monospace").child(value))
}

fn source_location(id: &InspectorElementId) -> String {
    let location = id.path.source_location;
    format!(
        "{}:{}:{}",
        location.file(),
        location.line(),
        location.column()
    )
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
}
