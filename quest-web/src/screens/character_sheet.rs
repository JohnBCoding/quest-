use yew::prelude::*;

use quest_core::action::{Action, ActionCondition, ActionTrigger};
use quest_core::player::Player;

#[derive(Properties, PartialEq)]
pub struct CharacterSheetProps {
    pub player: Player,
    pub on_close: Callback<()>,
    pub on_save_actions: Callback<Vec<Action>>,
}

fn trigger_label(trigger: &ActionTrigger) -> String {
    match trigger {
        ActionTrigger::EveryAction => "Every Action".to_string(),
        ActionTrigger::EveryNActions(n) => format!("Every {} Actions", n),
    }
}

fn action_condition_label(action: &Action) -> Option<String> {
    match action.condition {
        ActionCondition::None => None,
        ActionCondition::HealthBelowPercent(percent) => Some(format!("HP < {}%", percent)),
    }
}

fn action_detail_label(action: &Action, player: &Player) -> String {
    let mut parts = vec![trigger_label(&action.trigger)];
    if let Some(condition_label) = action_condition_label(action) {
        parts.push(condition_label);
    }
    if action.id == "health_potion" {
        parts.push(format!(
            "Uses {}/{}",
            player.health_potion_uses, player.health_potion_capacity
        ));
    }
    parts.join(" • ")
}

fn move_action(actions: &[Action], from: usize, to: usize) -> Vec<Action> {
    if from >= actions.len() || to >= actions.len() || from == to {
        return actions.to_vec();
    }

    let mut reordered = actions.to_vec();
    let action = reordered.remove(from);
    reordered.insert(to, action);
    reordered
}

fn render_action(action: &Action, player: &Player) -> Html {
    html! {
        <div class="action-row">
            <span class="action-name">{&action.name}</span>
            <span class="action-trigger">{action_detail_label(action, player)}</span>
        </div>
    }
}

#[function_component(CharacterSheetScreen)]
pub fn character_sheet_screen(props: &CharacterSheetProps) -> Html {
    let is_configuring = use_state(|| false);
    let draft_actions = use_state(|| props.player.actions.clone());
    let dragged_index = use_state(|| None::<usize>);

    let on_close = {
        let cb = props.on_close.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };

    let on_open_config = {
        let is_configuring = is_configuring.clone();
        let draft_actions = draft_actions.clone();
        let source_actions = props.player.actions.clone();
        Callback::from(move |_: MouseEvent| {
            draft_actions.set(source_actions.clone());
            is_configuring.set(true);
        })
    };

    let on_cancel_config = {
        let is_configuring = is_configuring.clone();
        let draft_actions = draft_actions.clone();
        let source_actions = props.player.actions.clone();
        Callback::from(move |_: MouseEvent| {
            draft_actions.set(source_actions.clone());
            is_configuring.set(false);
        })
    };

    let on_save_config = {
        let is_configuring = is_configuring.clone();
        let draft_actions = draft_actions.clone();
        let on_save_actions = props.on_save_actions.clone();
        Callback::from(move |_: MouseEvent| {
            on_save_actions.emit((*draft_actions).clone());
            is_configuring.set(false);
        })
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
                {
                    if *is_configuring {
                        let action_len = draft_actions.len();
                        html! {
                            <div class="actions-section">
                                <h2>{"Configure Actions"}</h2>
                                <p class="actions-description">
                                    {"Drag and drop actions to change combat priority (top runs first). On mobile, use the up/down buttons."}
                                </p>
                                <div class="actions-list configure-list">
                                    {
                                        if draft_actions.is_empty() {
                                            html! {
                                                <div class="action-row empty">
                                                    <span class="action-name">{"No actions configured"}</span>
                                                </div>
                                            }
                                        } else {
                                            html! {
                                                { for draft_actions.iter().enumerate().map(|(idx, action)| {
                                                    let dragged_index_state = dragged_index.clone();
                                                    let dragged_index_for_start = dragged_index_state.clone();
                                                    let ondragstart = Callback::from(move |e: DragEvent| {
                                                        if let Some(data) = e.data_transfer() {
                                                            let _ = data.set_data("text/plain", &idx.to_string());
                                                        }
                                                        dragged_index_for_start.set(Some(idx));
                                                    });

                                                    let ondragover = Callback::from(|e: DragEvent| {
                                                        e.prevent_default();
                                                    });

                                                    let dragged_index_for_drop = dragged_index_state.clone();
                                                    let draft_actions_for_drop = draft_actions.clone();
                                                    let ondrop = Callback::from(move |e: DragEvent| {
                                                        e.prevent_default();
                                                        if let Some(from) = *dragged_index_for_drop {
                                                            let next = move_action(draft_actions_for_drop.as_ref(), from, idx);
                                                            draft_actions_for_drop.set(next);
                                                            dragged_index_for_drop.set(None);
                                                        }
                                                    });

                                                    let draft_actions_for_up = draft_actions.clone();
                                                    let on_move_up = Callback::from(move |_: MouseEvent| {
                                                        if idx > 0 {
                                                            let next = move_action(draft_actions_for_up.as_ref(), idx, idx - 1);
                                                            draft_actions_for_up.set(next);
                                                        }
                                                    });

                                                    let draft_actions_for_down = draft_actions.clone();
                                                    let on_move_down = Callback::from(move |_: MouseEvent| {
                                                        if idx + 1 < action_len {
                                                            let next = move_action(draft_actions_for_down.as_ref(), idx, idx + 1);
                                                            draft_actions_for_down.set(next);
                                                        }
                                                    });

                                                    let dragged = *dragged_index_state == Some(idx);
                                                    html! {
                                                        <div
                                                            class={classes!("action-row", "action-row-draggable", if dragged { Some("dragging") } else { None })}
                                                            draggable={"true"}
                                                            {ondragstart}
                                                            {ondragover}
                                                            {ondrop}
                                                        >
                                                            <div class="action-main-info">
                                                                <span class="action-name">{format!("{} {}", idx + 1, action.name)}</span>
                                                                <span class="action-trigger">{action_detail_label(action, &props.player)}</span>
                                                            </div>
                                                            <div class="action-reorder-controls">
                                                                <button class="btn btn-secondary action-move-btn" onclick={on_move_up} disabled={idx == 0}>{"Up"}</button>
                                                                <button class="btn btn-secondary action-move-btn" onclick={on_move_down} disabled={idx + 1 >= action_len}>{"Down"}</button>
                                                            </div>
                                                        </div>
                                                    }
                                                })}
                                            }
                                        }
                                    }
                                </div>
                            </div>
                        }
                    } else {
                        html! {
                            <>
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
                                                    { for props.player.actions.iter().map(|a| render_action(a, &props.player)) }
                                                }
                                            }
                                        }
                                    </div>
                                    {
                                        if props.player.actions.is_empty() {
                                            html! {}
                                        } else {
                                            html! {
                                                <button class="btn btn-secondary configure-actions-cta" onclick={on_open_config}>
                                                    {"Configure"}
                                                </button>
                                            }
                                        }
                                    }
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
                            </>
                        }
                    }
                }
            </div>

            <div class="character-sheet-footer">
                {
                    if *is_configuring {
                        html! {
                            <div class="configure-footer-actions">
                                <button class="btn btn-primary" onclick={on_save_config}>
                                    {"Save"}
                                </button>
                                <button class="btn btn-secondary" onclick={on_cancel_config}>
                                    {"Cancel"}
                                </button>
                            </div>
                        }
                    } else {
                        html! {
                            <button class="btn btn-primary" onclick={on_close}>
                                {"Close"}
                            </button>
                        }
                    }
                }
            </div>
        </div>
    }
}
