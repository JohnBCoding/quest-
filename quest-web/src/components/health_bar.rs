use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct HealthBarProps {
    pub current: u32,
    pub max: u32,
    #[prop_or_default]
    pub label: Option<String>,
}

#[function_component(HealthBar)]
pub fn health_bar(props: &HealthBarProps) -> Html {
    let percent = if props.max > 0 {
        (props.current as f32 / props.max as f32) * 100.0
    } else {
        0.0
    };

    html! {
        <div class="hp-bar-container">
            <div class="hp-bar-fill" style={format!("width: {}%;", percent)}></div>
            <div class="hp-bar-text">
                {
                    if let Some(label) = &props.label {
                        format!("{}: {}/{}", label, props.current, props.max)
                    } else {
                        format!("{}/{}", props.current, props.max)
                    }
                }
            </div>
        </div>
    }
}
