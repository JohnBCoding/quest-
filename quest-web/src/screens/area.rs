use yew::prelude::*;

use quest_core::area::Area;
use quest_core::mob::Mob;
use quest_core::player::Player;
use crate::components::health_bar::HealthBar;

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
    pub on_mob_attack: Callback<()>,
    pub on_enter_portal: Callback<()>,
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

    // Boss spawn animation
    {
        let is_spawning_setter = is_spawning.clone();
        let is_boss = props.is_boss;
        let mob_id = props.current_mob.as_ref().map(|m| m.id.clone());
        use_effect_with(
            (is_boss, mob_id),
            move |_| {
                if is_boss {
                    is_spawning_setter.set(true);
                    let setter = is_spawning_setter.clone();
                    gloo_timers::callback::Timeout::new(1200, move || {
                        setter.set(false);
                    })
                    .forget();
                }
                || ()
            },
        );
    }

    // Portal spawn animation
    {
        let is_portal_spawning_setter = is_portal_spawning.clone();
        let can_show_portal = props.current_mob.is_none() && props.encounters_cleared >= props.area.base_encounter_amount;
        use_effect_with(
            can_show_portal,
            move |&can_show| {
                if can_show {
                    is_portal_spawning_setter.set(true);
                    let setter = is_portal_spawning_setter.clone();
                    gloo_timers::callback::Timeout::new(1000, move || {
                        setter.set(false);
                    })
                    .forget();
                }
                || ()
            }
        );
    }

    // Auto-combat action bar timer
    {
        let progress_state = action_progress.clone();
        let progress_ref = action_progress_ref.clone();
        let flash = action_flash.clone();
        let has_auto = props.has_auto_combat;
        let has_mob = props.current_mob.as_ref().map_or(false, |m| !m.is_dead());
        let action_speed = props.player.action_speed_ms;
        let on_attack_cb = props.on_attack.clone();
        let is_attacking_setter = is_attacking.clone();

        use_effect_with(
            (has_auto, has_mob, action_speed),
            move |(auto, mob_alive, speed)| {
                let mut interval_handle: Option<gloo_timers::callback::Interval> = None;

                if *auto && *mob_alive {
                    let tick_ms = 50u32;
                    let increment = (tick_ms as f64 / *speed as f64) * 100.0;
                    let ref_handle = progress_ref.clone();
                    let state_handle = progress_state.clone();
                    let flash_handle = flash.clone();
                    let attack_cb = on_attack_cb.clone();
                    let attacking_setter = is_attacking_setter.clone();

                    *ref_handle.borrow_mut() = 0.0;

                    interval_handle = Some(gloo_timers::callback::Interval::new(tick_ms, move || {
                        let mut val = ref_handle.borrow_mut();
                        *val += increment;
                        if *val >= 100.0 {
                            flash_handle.set(true);
                            
                            // Trigger attack animation
                            attacking_setter.set(true);
                            let anim_reset = attacking_setter.clone();
                            gloo_timers::callback::Timeout::new(400, move || {
                                anim_reset.set(false);
                            })
                            .forget();
                            
                            attack_cb.emit(());
                            *val = 0.0;
                            state_handle.set(0.0);
                            let flash_reset = flash_handle.clone();
                            gloo_timers::callback::Timeout::new(300, move || {
                                flash_reset.set(false);
                            })
                            .forget();
                        } else {
                            state_handle.set(*val);
                        }
                    }));
                } else {
                    *progress_ref.borrow_mut() = 0.0;
                    progress_state.set(0.0);
                }

                move || drop(interval_handle)
            },
        );
    }

    // Mob auto-attack timer
    {
        let has_mob = props.current_mob.as_ref().map_or(false, |m| !m.is_dead());
        let player_alive = props.player.is_alive();
        let mob_action_speed = props.current_mob.as_ref().map(|m| m.action_speed_ms).unwrap_or(0);
        let on_mob_attack_cb = props.on_mob_attack.clone();
        let player_hit_setter = player_hit.clone();
        let mob_progress_state = mob_action_progress.clone();
        let mob_progress_ref = mob_action_progress_ref.clone();
        let mob_flash = mob_action_flash.clone();

        use_effect_with(
            (has_mob, player_alive, mob_action_speed),
            move |(mob_alive, alive, speed)| {
                let mut interval_handle: Option<gloo_timers::callback::Interval> = None;

                if *mob_alive && *alive {
                    let tick_ms = 50u32;
                    let speed_ms = if *speed == 0 { 1000 } else { *speed };
                    let increment = (tick_ms as f64 / speed_ms as f64) * 100.0;
                    let ref_handle = mob_progress_ref.clone();
                    let state_handle = mob_progress_state.clone();
                    let flash_handle = mob_flash.clone();
                    let attack_cb = on_mob_attack_cb.clone();
                    let hit_setter = player_hit_setter.clone();

                    *ref_handle.borrow_mut() = 0.0;

                    interval_handle = Some(gloo_timers::callback::Interval::new(tick_ms, move || {
                        let mut val = ref_handle.borrow_mut();
                        *val += increment;
                        if *val >= 100.0 {
                            flash_handle.set(true);
                            hit_setter.set(true);
                            let reset = hit_setter.clone();
                            gloo_timers::callback::Timeout::new(400, move || {
                                reset.set(false);
                            })
                            .forget();
                            attack_cb.emit(());
                            *val = 0.0;
                            state_handle.set(0.0);
                            let flash_reset = flash_handle.clone();
                            gloo_timers::callback::Timeout::new(300, move || {
                                flash_reset.set(false);
                            })
                            .forget();
                        } else {
                            state_handle.set(*val);
                        }
                    }));
                } else {
                    *mob_progress_ref.borrow_mut() = 0.0;
                    mob_progress_state.set(0.0);
                }

                move || drop(interval_handle)
            },
        );
    }

    let on_exit = {
        let cb = props.on_exit.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };

    let on_enter_portal = {
        let cb = props.on_enter_portal.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };

    let on_attack = {
        let cb = props.on_attack.clone();
        let is_attacking_setter = is_attacking.clone();
        Callback::from(move |_: MouseEvent| {
            is_attacking_setter.set(true);
            cb.emit(());
            let setter = is_attacking_setter.clone();
            gloo_timers::callback::Timeout::new(400, move || {
                setter.set(false);
            })
            .forget();
        })
    };

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
                        let anim_class = if *is_attacking {
                            "animating-attack"
                        } else if mob.is_dead() {
                            "dead"
                        } else {
                            ""
                        };

                        let boss_class = if props.is_boss { "boss-encounter" } else { "" };
                        let spawn_class = if *is_spawning { "spawning-boss" } else { "" };

                        html! {
                            <div class={classes!("mob-hud", anim_class, boss_class, spawn_class)}>
                                <h3>{ &mob.name }</h3>
                                <HealthBar
                                    current={mob.health}
                                    max={mob.max_health}
                                    label={Some("HP".to_string())}
                                />
                                {
                                    if !mob.is_dead() {
                                        let flash_class = if *mob_action_flash { "action-speed-bar-flash" } else { "" };
                                        html! {
                                            <div class={classes!("action-speed-bar-container", flash_class)}>
                                                <div class="action-speed-bar-fill" style={format!("width: {}%;", *mob_action_progress)}></div>
                                                <div class="action-speed-bar-text">{"Action"}</div>
                                            </div>
                                        }
                                    } else {
                                        html! {}
                                    }
                                }
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
                <div class={classes!("player-hud", if *player_hit { "player-hit" } else { "" })}>
                    <div class="player-name">{ &props.player.name }</div>
                    <HealthBar
                        current={props.player.health}
                        max={props.player.max_health}
                        label={Some("HP".to_string())}
                    />
                    {
                        if props.has_auto_combat {
                            let flash_class = if *action_flash { "action-speed-bar-flash" } else { "" };
                            html! {
                                <div class={classes!("action-speed-bar-container", flash_class)}>
                                    <div class="action-speed-bar-fill" style={format!("width: {}%;", *action_progress)}></div>
                                    <div class="action-speed-bar-text">{"Action"}</div>
                                </div>
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
                    <button class="btn btn-danger" onclick={on_exit}>
                        { "Exit Game" }
                    </button>
                </div>
            </div>
        </div>
    }
}
