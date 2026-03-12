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
    pub on_exit: Callback<()>,
    pub on_attack: Callback<()>,
    pub on_enter_portal: Callback<()>,
}

#[function_component(AreaScreen)]
pub fn area_screen(props: &AreaScreenProps) -> Html {
    let is_attacking = use_state(|| false);
    let is_spawning = use_state(|| false);
    let is_portal_spawning = use_state(|| false);

    // Trigger spawn animation when a boss appears
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

    // Trigger portal spawn animation
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
                    <div class="player-name">{ &props.player.name }</div>
                    <HealthBar 
                        current={props.player.health} 
                        max={props.player.max_health} 
                        label={Some("HP".to_string())} 
                    />
                </div>
                <div class="action-buttons">
                    {
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
                    }
                    <button class="btn btn-danger" onclick={on_exit}>
                        { "Exit Game" }
                    </button>
                </div>
            </div>
        </div>
    }
}
