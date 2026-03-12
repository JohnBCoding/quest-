use yew::prelude::*;

use quest_core::fruit::Fruit;

#[derive(Properties, PartialEq, Clone)]
pub struct FruitSceneProps {
    pub fruit_id: String,
    pub on_eat_fruit: Callback<()>,
}

#[function_component(FruitSceneScreen)]
pub fn fruit_scene_screen(props: &FruitSceneProps) -> Html {
    let phase = use_state(|| 0u8);
    let fruit = Fruit::get_by_id(&props.fruit_id);

    let fruit_name = fruit
        .as_ref()
        .map(|f| f.name.clone())
        .unwrap_or_else(|| "Unknown Fruit".to_string());

    let fruit_description = fruit
        .as_ref()
        .map(|f| f.description.clone())
        .unwrap_or_default();

    // Auto-advance phases 0→1→2 only
    {
        let phase_setter = phase.clone();
        let current_phase = *phase;
        use_effect_with(current_phase, move |&p| {
            if p < 2 {
                let delay = if p == 0 { 3500 } else { 4500 };
                let setter = phase_setter.clone();
                gloo_timers::callback::Timeout::new(delay, move || {
                    setter.set(p + 1);
                })
                .forget();
            }
            || ()
        });
    }

    // Phase 3: after eating, show description then fire callback
    {
        let phase_val = *phase;
        let cb = props.on_eat_fruit.clone();
        use_effect_with(phase_val, move |&p| {
            if p == 3 {
                let callback = cb.clone();
                gloo_timers::callback::Timeout::new(3500, move || {
                    callback.emit(());
                })
                .forget();
            }
            || ()
        });
    }

    let on_eat = {
        let phase_setter = phase.clone();
        Callback::from(move |_: MouseEvent| {
            phase_setter.set(3);
        })
    };

    html! {
        <div class="screen fruit-scene">
            <div class="fruit-scene-backdrop">
                <div class={classes!("fruit-scene-content", match *phase {
                    0 => "phase-drop",
                    1 => "phase-wonder",
                    2 => "phase-choice",
                    _ => "phase-eaten",
                })}>
                    {
                        match *phase {
                            0 => html! {
                                <>
                                    <div class="fruit-drop-animation">
                                        <div class="fruit-icon fruit-glow">{"🍎"}</div>
                                    </div>
                                    <p class="scene-text scene-text-fade">
                                        {"Something falls from the creature..."}
                                    </p>
                                </>
                            },
                            1 => html! {
                                <>
                                    <div class="fruit-icon fruit-glow fruit-landed">{"🍎"}</div>
                                    <p class="scene-text scene-text-fade">
                                        {"A strange fruit pulses with energy. You wonder what it could be..."}
                                    </p>
                                </>
                            },
                            2 => html! {
                                <>
                                    <div class="fruit-icon fruit-glow fruit-ready">{"🍎"}</div>
                                    <h2 class="fruit-name">{&fruit_name}</h2>
                                    <button class="btn btn-fruit-eat" onclick={on_eat}>
                                        {"Eat the Fruit"}
                                    </button>
                                </>
                            },
                            _ => html! {
                                <>
                                    <div class="fruit-icon fruit-consumed">{"🍎"}</div>
                                    <h2 class="fruit-name">{&fruit_name}</h2>
                                    <p class="fruit-desc">{&fruit_description}</p>
                                </>
                            },
                        }
                    }
                </div>
            </div>
        </div>
    }
}
