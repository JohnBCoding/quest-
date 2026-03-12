use yew::prelude::*;

use quest_core::area::Area;
use quest_core::mob::Mob;
use quest_core::player::Player;

#[derive(Properties, PartialEq)]
pub struct AreaScreenProps {
    pub area: Area,
    pub player: Player,
    pub current_mob: Option<Mob>,
    pub encounters_cleared: u32,
    pub on_exit: Callback<()>,
    pub on_attack: Callback<()>,
}

#[function_component(AreaScreen)]
pub fn area_screen(props: &AreaScreenProps) -> Html {
    let is_attacking = use_state(|| false);

    let on_exit = {
        let cb = props.on_exit.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };

    let on_attack = {
        let cb = props.on_attack.clone();
        let is_attacking_setter = is_attacking.clone();
        Callback::from(move |_: MouseEvent| {
            is_attacking_setter.set(true);
            cb.emit(());
            let setter = is_attacking_setter.clone();
            gloo_timers::callback::Timeout::new(200, move || {
                setter.set(false);
            }).forget();
        })
    };

    html! {
        <div class="screen area-screen">
            <div class="area-header">
                <div class="area-name">{ &props.area.name }</div>
                <div class="player-info">
                    <span class="player-name">{ &props.player.name }</span>
                    <span class="player-hp">
                        { format!("HP {}/{}", props.player.health, props.player.max_health) }
                    </span>
                </div>
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
                        let hp_percent = if mob.max_health > 0 {
                            (mob.health as f32 / mob.max_health as f32) * 100.0
                        } else {
                            0.0
                        };
                        
                        let anim_class = if *is_attacking {
                            "animating-attack"
                        } else if mob.is_dead() {
                            "dead"
                        } else {
                            ""
                        };

                        html! {
                            <div class={classes!("mob-hud", anim_class)}>
                                <h3>{ &mob.name }</h3>
                                <div>
                                    <div class="hp-bar-container">
                                        <div class="hp-bar-fill" style={format!("width: {}%;", hp_percent)}></div>
                                    </div>
                                    <div class="mob-hp-text">
                                        { format!("HP: {}/{}", mob.health, mob.max_health) }
                                    </div>
                                </div>
                                <button class="btn btn-primary" onclick={on_attack.clone()} disabled={mob.is_dead() || *is_attacking}>
                                    { "Attack" }
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
                <button class="btn btn-danger" onclick={on_exit}>
                    { "Exit Game" }
                </button>
            </div>
        </div>
    }
}
