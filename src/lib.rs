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
            .child(render_element_id(&id, config))
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
                        .px_2()
                        .py_1()
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

fn render_element_id(id: &InspectorElementId, config: &Config) -> Div {
    section("Selected element", config)
        .child(property("Source", source_location(id), config))
        .child(
            div()
                .flex()
                .gap_3()
                .child(property("Instance", id.instance_id.to_string(), config))
                .child(property("Global ID", id.path.global_id.to_string(), config)),
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
        .child(section("Style", config).child(property(
            "Refinement",
            format!("{:#?}", state.base_style),
            config,
        )))
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
    fn source_paths_are_compact() {
        assert_eq!(
            compact_source_path("/workspace/app/src/views/card.rs"),
            "src/views/card.rs"
        );
        assert_eq!(compact_source_path("main.rs"), "main.rs");
    }
}
