use yew::prelude::*;

use crate::app::PlayerActionKind;
use crate::components::health_bar::HealthBar;
use quest_core::area::Area;
use quest_core::mob::Mob;
use quest_core::player::Player;

#[derive(Properties, PartialEq)]
pub struct AreaScreenProps {
    pub area: Area,
    pub player: Player,
    pub current_mob: Option<Mob>,
    pub encounters_cleared: u32,
    pub is_boss: bool,
    pub has_auto_combat: bool,
    pub on_exit: Callback<()>,
    pub on_attack: Callback<()>,
    pub on_auto_action: Callback<()>,
    pub on_mob_attack: Callback<()>,
    pub on_enter_portal: Callback<()>,
    pub on_portal_to_town: Callback<()>,
    pub can_portal_to_town: bool,
    pub is_portal_to_town_pending: bool,
    pub action_progress_reset_event_id: u64,
    pub is_portal_to_town_transitioning: bool,
    pub last_player_action_kind: Option<PlayerActionKind>,
    pub player_action_event_id: u64,
    pub mob_action_event_id: u64,
}

#[function_component(AreaScreen)]
pub fn area_screen(props: &AreaScreenProps) -> Html {
    let is_attacking = use_state(|| false);
    let is_spawning = use_state(|| false);
    let is_portal_spawning = use_state(|| false);
    let action_progress = use_state(|| 0.0f64);
    let action_progress_ref = use_mut_ref(|| 0.0f64);
    let action_flash = use_state(|| false);
    let mob_action_progress = use_state(|| 0.0f64);
    let mob_action_progress_ref = use_mut_ref(|| 0.0f64);
    let mob_action_flash = use_state(|| false);
    let player_hit = use_state(|| false);
    let player_heal = use_state(|| false);
    let level_up_flash = use_state(|| false);
    let prev_player_level = use_mut_ref(|| props.player.level);

    // Boss spawn animation
    {
        let is_spawning_setter = is_spawning.clone();
        let is_boss = props.is_boss;
        let mob_id = props.current_mob.as_ref().map(|m| m.id.clone());
        use_effect_with((is_boss, mob_id), move |_| {
            if is_boss {
                is_spawning_setter.set(true);
                let setter = is_spawning_setter.clone();
                gloo_timers::callback::Timeout::new(1200, move || {
                    setter.set(false);
                })
                .forget();
            }
            || ()
        });
    }

    // Portal spawn animation
    {
        let is_portal_spawning_setter = is_portal_spawning.clone();
        let can_show_portal = props.current_mob.is_none()
            && props.encounters_cleared >= props.area.base_encounter_amount;
        use_effect_with(can_show_portal, move |&can_show| {
            if can_show {
                is_portal_spawning_setter.set(true);
                let setter = is_portal_spawning_setter.clone();
                gloo_timers::callback::Timeout::new(1000, move || {
                    setter.set(false);
                })
                .forget();
            }
            || ()
        });
    }

    // Player action animation trigger from app event stream
    {
        let is_attacking_setter = is_attacking.clone();
        let player_heal_setter = player_heal.clone();
        let action_event_id = props.player_action_event_id;
        let action_kind = props.last_player_action_kind.clone();

        use_effect_with((action_event_id, action_kind), move |(event_id, kind)| {
            if *event_id != 0 {
                match kind {
                    Some(PlayerActionKind::Attack) => {
                        is_attacking_setter.set(true);
                        let reset = is_attacking_setter.clone();
                        gloo_timers::callback::Timeout::new(400, move || {
                            reset.set(false);
                        })
                        .forget();
                    }
                    Some(PlayerActionKind::HealPotion) => {
                        player_heal_setter.set(true);
                        let reset = player_heal_setter.clone();
                        gloo_timers::callback::Timeout::new(500, move || {
                            reset.set(false);
                        })
                        .forget();
                    }
                    None => {}
                }
            }

            || ()
        });
    }

    // Mob hit animation trigger from app event stream
    {
        let event_id = props.mob_action_event_id;
        let player_hit_setter = player_hit.clone();

        use_effect_with(event_id, move |event_id| {
            if *event_id != 0 {
                player_hit_setter.set(true);
                let reset = player_hit_setter.clone();
                gloo_timers::callback::Timeout::new(400, move || {
                    reset.set(false);
                })
                .forget();
            }

            || ()
        });
    }

    // Combat loop timer (player and mob bars share one tick stream)
    {
        let has_auto = props.has_auto_combat;
        let has_mob = props.current_mob.as_ref().map_or(false, |m| !m.is_dead());
        let player_alive = props.player.is_alive();
        let is_portal_transitioning = props.is_portal_to_town_transitioning;
        let player_action_speed = props.player.action_speed_ms;
        let mob_action_speed = props
            .current_mob
            .as_ref()
            .map(|m| m.action_speed_ms)
            .unwrap_or(0);
        let on_player_action_cb = props.on_auto_action.clone();
        let on_mob_attack_cb = props.on_mob_attack.clone();
        let player_progress_state = action_progress.clone();
        let player_progress_ref = action_progress_ref.clone();
        let player_flash = action_flash.clone();
        let mob_progress_state = mob_action_progress.clone();
        let mob_progress_ref = mob_action_progress_ref.clone();
        let mob_flash = mob_action_flash.clone();

        use_effect_with(
            (
                has_auto,
                has_mob,
                player_alive,
                is_portal_transitioning,
                player_action_speed,
                mob_action_speed,
            ),
            move |(auto, mob_alive, alive, portal_transitioning, player_speed, mob_speed)| {
                let mut interval_handle: Option<gloo_timers::callback::Interval> = None;

                if *mob_alive && *alive && !*portal_transitioning {
                    let auto_enabled = *auto;
                    let tick_ms = 50u32;
                    let player_speed_ms = if *player_speed == 0 {
                        1000
                    } else {
                        *player_speed
                    };
                    let player_increment = (tick_ms as f64 / player_speed_ms as f64) * 100.0;
                    let mob_speed_ms = if *mob_speed == 0 { 1000 } else { *mob_speed };
                    let mob_increment = (tick_ms as f64 / mob_speed_ms as f64) * 100.0;
                    let player_ref = player_progress_ref.clone();
                    let player_state = player_progress_state.clone();
                    let player_flash_handle = player_flash.clone();
                    let mob_ref = mob_progress_ref.clone();
                    let mob_state = mob_progress_state.clone();
                    let mob_flash_handle = mob_flash.clone();
                    let player_action_cb = on_player_action_cb.clone();
                    let mob_attack_cb = on_mob_attack_cb.clone();

                    *player_ref.borrow_mut() = 0.0;
                    *mob_ref.borrow_mut() = 0.0;

                    interval_handle =
                        Some(gloo_timers::callback::Interval::new(tick_ms, move || {
                            let mut player_fired_this_tick = false;

                            if auto_enabled {
                                let mut player_val = player_ref.borrow_mut();
                                *player_val += player_increment;
                                if *player_val >= 100.0 {
                                    player_flash_handle.set(true);
                                    player_action_cb.emit(());
                                    *player_val = 0.0;
                                    player_state.set(0.0);
                                    player_fired_this_tick = true;
                                    let flash_reset = player_flash_handle.clone();
                                    gloo_timers::callback::Timeout::new(300, move || {
                                        flash_reset.set(false);
                                    })
                                    .forget();
                                } else {
                                    player_state.set(*player_val);
                                }
                            } else {
                                *player_ref.borrow_mut() = 0.0;
                                player_state.set(0.0);
                            }

                            let mut mob_val = mob_ref.borrow_mut();
                            *mob_val += mob_increment;
                            if *mob_val >= 100.0 {
                                let delay_ms = if player_fired_this_tick { 450 } else { 0 };
                                let flash = mob_flash_handle.clone();
                                let cb = mob_attack_cb.clone();
                                let execute_mob = move || {
                                    flash.set(true);
                                    cb.emit(());
                                    let flash_reset = flash.clone();
                                    gloo_timers::callback::Timeout::new(300, move || {
                                        flash_reset.set(false);
                                    })
                                    .forget();
                                };

                                if delay_ms > 0 {
                                    gloo_timers::callback::Timeout::new(delay_ms, execute_mob)
                                        .forget();
                                } else {
                                    execute_mob();
                                }

                                *mob_val = 0.0;
                                mob_state.set(0.0);
                            } else {
                                mob_state.set(*mob_val);
                            }
                        }));
                } else {
                    *player_progress_ref.borrow_mut() = 0.0;
                    player_progress_state.set(0.0);
                    *mob_progress_ref.borrow_mut() = 0.0;
                    mob_progress_state.set(0.0);
                    if *portal_transitioning {
                        player_flash.set(false);
                        mob_flash.set(false);
                    }
                }

                move || drop(interval_handle)
            },
        );
    }

    // Reset action timer when the player queues portal-to-town.
    {
        let reset_event_id = props.action_progress_reset_event_id;
        let player_progress_state = action_progress.clone();
        let player_progress_ref = action_progress_ref.clone();
        let player_flash_state = action_flash.clone();

        use_effect_with(reset_event_id, move |event_id| {
            if *event_id != 0 {
                *player_progress_ref.borrow_mut() = 0.0;
                player_progress_state.set(0.0);
                player_flash_state.set(false);
            }
            || ()
        });
    }

    // Level-up HUD animation trigger
    {
        let flash_state = level_up_flash.clone();
        let prev_level_ref = prev_player_level.clone();
        let current_level = props.player.level;

        use_effect_with(current_level, move |level| {
            let mut prev = prev_level_ref.borrow_mut();
            if *level > *prev {
                flash_state.set(true);
                let flash_reset = flash_state.clone();
                gloo_timers::callback::Timeout::new(900, move || {
                    flash_reset.set(false);
                })
                .forget();
            }
            *prev = *level;
            || ()
        });
    }

    let on_exit = {
        let cb = props.on_exit.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };

    let on_enter_portal = {
        let cb = props.on_enter_portal.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };

    let on_portal_to_town = {
        let cb = props.on_portal_to_town.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };

    let on_attack = {
        let cb = props.on_attack.clone();
        Callback::from(move |_: MouseEvent| {
            cb.emit(());
        })
    };

    let xp_progress_pct = if props.player.max_experience == 0 {
        0.0
    } else {
        ((props.player.experience as f64 / props.player.max_experience as f64) * 100.0)
            .clamp(0.0, 100.0)
    };
    let has_health_potion_action = props
        .player
        .actions
        .iter()
        .any(|action| action.id == "health_potion");

    html! {
        <div class="screen area-screen">
            <div class="area-header">
                <div class="area-name">{ &props.area.name }</div>
            </div>

            <div class="area-body">
                <div class="area-description">
                    <p>{ &props.area.description }</p>
                    <p class="encounter-progress">
                        { format!("Encounters Cleared: {}/{}", props.encounters_cleared, props.area.base_encounter_amount) }
                    </p>
                </div>

                {
                    if let Some(mob) = &props.current_mob {
                        let hit_class = if *is_attacking { "animating-attack" } else { "" };
                        let dead_class = if mob.is_dead() { "dead" } else { "" };
                        let boss_class = if props.is_boss { "boss-encounter" } else { "" };
                        let spawn_class = if *is_spawning { "spawning-boss" } else { "" };
                        let mob_flash_class = if *mob_action_flash { "action-speed-bar-flash" } else { "" };
                        html! {
                            <div class={classes!("mob-hud", dead_class, boss_class, spawn_class)}>
                                <div class={classes!("mob-vitals", hit_class)}>
                                    <h3>{ &mob.name }</h3>
                                    <HealthBar
                                        current={mob.health}
                                        max={mob.max_health}
                                        label={Some("HP".to_string())}
                                    />
                                </div>
                                <div class={classes!("action-speed-bar-container", mob_flash_class)}>
                                    <div class="action-speed-bar-fill" style={format!("width: {}%;", *mob_action_progress)}></div>
                                    <div class="action-speed-bar-text">{"Action"}</div>
                                </div>
                            </div>
                        }
                    } else if props.encounters_cleared >= props.area.base_encounter_amount {
                        let portal_anim_class = if *is_portal_spawning { "portal-entrance" } else { "" };
                        html! {
                            <div class={classes!("area-cleared", "portal-spawn", portal_anim_class)}>
                                <div class="portal-shimmer"></div>
                                <p>{ "The air shimmers with dark energy..." }</p>
                                <button class="btn btn-warning" onclick={on_enter_portal}>
                                    { "Enter Mysterious Portal" }
                                </button>
                            </div>
                        }
                    } else {
                        html! {
                            <div class="area-cleared">
                                <p>{ "Area Cleared!" }</p>
                            </div>
                        }
                    }
                }
            </div>

            <div class="action-bar">
                <div class="player-hud">
                    <div class="player-header">
                        <div class="player-name-row">
                            <div class="player-name">{ &props.player.name }</div>
                            {
                                if has_health_potion_action {
                                    html! {
                                        <div class="player-potion-hud">
                                            <span class="potion-hud-icon">{"🧪"}</span>
                                            <span class="potion-hud-count">
                                                {format!("{}/{}", props.player.health_potion_uses, props.player.health_potion_capacity)}
                                            </span>
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }
                            }
                        </div>
                        <div class="player-level-block">
                            <div class={classes!("player-level", if *level_up_flash { "level-up-flash" } else { "" })}>
                                { format!("LV {}", props.player.level) }
                            </div>
                            <div class={classes!("player-exp-bar", if *level_up_flash { "level-up-flash" } else { "" })}>
                                <div
                                    class="player-exp-bar-fill"
                                    style={format!("width: {:.2}%;", xp_progress_pct)}
                                />
                            </div>
                        </div>
                    </div>
                    <div class={classes!(
                        "player-vitals",
                        if *player_hit { Some("player-hit") } else { None },
                        if *player_heal { Some("player-heal") } else { None }
                    )}>
                        <HealthBar
                            current={props.player.health}
                            max={props.player.max_health}
                            label={Some("HP".to_string())}
                        />
                    </div>
                    {
                        if props.has_auto_combat {
                            if props.is_portal_to_town_pending {
                                html! {
                                    <div class="portal-action-speed-bar-container">
                                        <div class="portal-action-speed-bar-fill" style={format!("width: {}%;", *action_progress)}></div>
                                        <div class="portal-action-speed-bar-shimmer"></div>
                                        <div class="portal-action-speed-bar-text">{"Portal"}</div>
                                    </div>
                                }
                            } else {
                                let flash_class = if *action_flash { "action-speed-bar-flash" } else { "" };
                                html! {
                                    <div class={classes!("action-speed-bar-container", flash_class)}>
                                        <div class="action-speed-bar-fill" style={format!("width: {}%;", *action_progress)}></div>
                                        <div class="action-speed-bar-text">{"Action"}</div>
                                    </div>
                                }
                            }
                        } else {
                            html! {}
                        }
                    }
                </div>
                <div class="action-buttons">
                    {
                        if !props.has_auto_combat {
                            if let Some(mob) = &props.current_mob {
                                html! {
                                    <button class="btn btn-primary" onclick={on_attack.clone()} disabled={mob.is_dead() || *is_attacking}>
                                        { "Attack" }
                                    </button>
                                }
                            } else {
                                html! {
                                    <button class="btn btn-primary" disabled=true>
                                        { "Attack" }
                                    </button>
                                }
                            }
                        } else {
                            html! {}
                        }
                    }
                    {
                        if props.can_portal_to_town {
                            html! {
                                <button class="btn btn-warning" onclick={on_portal_to_town} disabled={props.is_portal_to_town_pending}>
                                    {
                                        if props.is_portal_to_town_pending {
                                            "Portaling..."
                                        } else {
                                            "Portal To Town"
                                        }
                                    }
                                </button>
                            }
                        } else {
                            html! {
                                <button class="btn btn-danger" onclick={on_exit}>
                                    { "Exit Game" }
                                </button>
                            }
                        }
                    }
                </div>
            </div>
        </div>
    }
}
