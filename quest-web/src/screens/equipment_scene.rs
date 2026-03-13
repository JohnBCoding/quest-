use yew::prelude::*;

use quest_core::equipment::EquipmentItem;

#[derive(Properties, PartialEq, Clone)]
pub struct EquipmentSceneProps {
    pub item_id: String,
    pub on_equip_item: Callback<()>,
}

#[function_component(EquipmentSceneScreen)]
pub fn equipment_scene_screen(props: &EquipmentSceneProps) -> Html {
    let phase = use_state(|| 0u8);

    let initial_item_id = use_state({
        let id = props.item_id.clone();
        move || id
    });

    let item = EquipmentItem::get_by_id(&initial_item_id);
    let item_name = item
        .as_ref()
        .map(|item| item.name.clone())
        .unwrap_or_else(|| "Unknown Item".to_string());
    let item_description = item
        .as_ref()
        .map(|item| item.description.clone())
        .unwrap_or_default();

    {
        let phase_setter = phase.clone();
        let current_phase = *phase;
        use_effect_with(current_phase, move |&p| {
            if p == 0 {
                let setter = phase_setter.clone();
                gloo_timers::callback::Timeout::new(3000, move || setter.set(1)).forget();
            }
            || ()
        });
    }

    {
        let phase_value = *phase;
        let cb = props.on_equip_item.clone();
        use_effect_with(phase_value, move |&p| {
            if p == 2 {
                let callback = cb.clone();
                gloo_timers::callback::Timeout::new(3600, move || callback.emit(())).forget();
            }
            || ()
        });
    }

    let on_equip = {
        let phase_setter = phase.clone();
        Callback::from(move |_: MouseEvent| phase_setter.set(2))
    };

    html! {
        <div class="screen fruit-scene">
            <div class="fruit-scene-backdrop">
                <div class={classes!("fruit-scene-content", match *phase {
                    0 => "phase-drop",
                    1 => "phase-choice",
                    _ => "phase-eaten",
                })}>
                    {
                        match *phase {
                            0 => html! {
                                <>
                                    <div class="fruit-drop-animation">
                                        <div class="fruit-icon">{"🗡️"}</div>
                                    </div>
                                    <p class="scene-text scene-text-fade">{"A blade skids across the ground."}</p>
                                </>
                            },
                            1 => html! {
                                <>
                                    <div class="fruit-icon fruit-ready">{"🗡️"}</div>
                                    <h2 class="fruit-name">{&item_name}</h2>
                                    <button class="btn btn-fruit-eat" onclick={on_equip}>{"Equip"}</button>
                                </>
                            },
                            _ => html! {
                                <>
                                    <div class="fruit-icon fruit-consumed">{"🗡️"}</div>
                                    <h2 class="fruit-name">{"Now we're talking."}</h2>
                                    <p class="fruit-desc">{item_description}</p>
                                </>
                            },
                        }
                    }
                </div>
            </div>
        </div>
    }
}
