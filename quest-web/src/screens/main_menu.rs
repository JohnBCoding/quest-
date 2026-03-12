use yew::prelude::*;

use crate::storage;

#[derive(Properties, PartialEq)]
pub struct MainMenuProps {
    pub on_new_game: Callback<()>,
    pub on_load_game: Callback<()>,
}

#[function_component(MainMenuScreen)]
pub fn main_menu(props: &MainMenuProps) -> Html {
    let has_save = storage::has_valid_save();

    let on_new = {
        let cb = props.on_new_game.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };

    let on_load = {
        let cb = props.on_load_game.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };

    html! {
        <div class="screen main-menu">
            <div class="menu-content">
                <div class="title-container">
                    <h1 class="game-title">{ "Quest!" }</h1>
                    <div class="title-underline" />
                </div>
                <div class="menu-buttons">
                    <button class="btn btn-primary" onclick={on_new}>
                        { "New Game" }
                    </button>
                    if has_save {
                        <button class="btn btn-secondary" onclick={on_load}>
                            { "Load Game" }
                        </button>
                    }
                </div>
            </div>
            <div class="menu-footer">
                <span class="version-text">{ "v0.1.0" }</span>
            </div>
        </div>
    }
}
