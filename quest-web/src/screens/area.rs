use yew::prelude::*;

use quest_core::area::Area;
use quest_core::player::Player;

#[derive(Properties, PartialEq)]
pub struct AreaScreenProps {
    pub area: Area,
    pub player: Player,
    pub on_exit: Callback<()>,
}

#[function_component(AreaScreen)]
pub fn area_screen(props: &AreaScreenProps) -> Html {
    let on_exit = {
        let cb = props.on_exit.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
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
                </div>
            </div>

            <div class="action-bar">
                <button class="btn btn-danger" onclick={on_exit}>
                    { "Exit Game" }
                </button>
            </div>
        </div>
    }
}
