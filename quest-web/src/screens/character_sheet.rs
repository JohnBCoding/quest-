use yew::prelude::*;

use quest_core::action::{Action, ActionTrigger};
use quest_core::player::Player;

#[derive(Properties, PartialEq)]
pub struct CharacterSheetProps {
    pub player: Player,
    pub on_close: Callback<()>,
}

fn trigger_label(trigger: &ActionTrigger) -> String {
    match trigger {
        ActionTrigger::EveryAction => "Every Action".to_string(),
        ActionTrigger::EveryNActions(n) => format!("Every {} Actions", n),
    }
}

fn render_action(action: &Action) -> Html {
    html! {
        <div class="action-row">
            <span class="action-name">{&action.name}</span>
            <span class="action-trigger">{trigger_label(&action.trigger)}</span>
        </div>
    }
}

#[function_component(CharacterSheetScreen)]
pub fn character_sheet_screen(props: &CharacterSheetProps) -> Html {
    let on_close = {
        let cb = props.on_close.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };

    html! {
        <div class="screen character-sheet">
            <div class="character-sheet-header">
                <h1>{"Character Sheet"}</h1>
                <div class="player-info">
                    <span class="player-stat">{format!("{} — Lv. {}", props.player.name, props.player.level)}</span>
                </div>
            </div>

            <div class="character-sheet-body">
                <div class="actions-section">
                    <h2>{"Actions"}</h2>
                    <p class="actions-description">
                        {"Your character will automatically perform these actions in combat."}
                    </p>
                    <div class="actions-list">
                        {
                            if props.player.actions.is_empty() {
                                html! {
                                    <div class="action-row empty">
                                        <span class="action-name">{"No actions configured"}</span>
                                    </div>
                                }
                            } else {
                                html! {
                                    { for props.player.actions.iter().map(render_action) }
                                }
                            }
                        }
                    </div>
                </div>

                <div class="stats-section">
                    <h2>{"Stats"}</h2>
                    <div class="stat-row">
                        <span class="stat-label">{"Action Speed"}</span>
                        <span class="stat-value">{format!("{:.1}s", props.player.action_speed_ms as f32 / 1000.0)}</span>
                    </div>
                    <div class="stat-row">
                        <span class="stat-label">{"HP"}</span>
                        <span class="stat-value">{format!("{}/{}", props.player.health, props.player.max_health)}</span>
                    </div>
                </div>

                <div class="fruits-section">
                    <h2>{"Eaten Fruits"}</h2>
                    <div class="fruits-list">
                        {
                            if props.player.eaten_fruits.is_empty() {
                                html! { <span class="no-fruits">{"None"}</span> }
                            } else {
                                html! {
                                    { for props.player.eaten_fruits.iter().map(|f| {
                                        let fruit = quest_core::fruit::Fruit::get_by_id(f);
                                        let name = fruit.as_ref().map(|fr| fr.name.clone()).unwrap_or_else(|| f.clone());
                                        html! {
                                            <div class="fruit-badge">
                                                <span class="fruit-badge-icon">{"🍎"}</span>
                                                <span class="fruit-badge-name">{name}</span>
                                            </div>
                                        }
                                    })}
                                }
                            }
                        }
                    </div>
                </div>
            </div>

            <div class="character-sheet-footer">
                <button class="btn btn-primary" onclick={on_close}>
                    {"Close"}
                </button>
            </div>
        </div>
    }
}
