use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TownScreenProps {
    pub has_auto_combat: bool,
    pub on_exit: Callback<()>,
    pub on_open_character_sheet: Callback<()>,
    pub on_open_inventory: Callback<()>,
    pub on_travel_dying_forest: Callback<()>,
}

#[function_component(TownScreen)]
pub fn town_screen(props: &TownScreenProps) -> Html {
    let on_exit = {
        let cb = props.on_exit.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };

    let on_character_sheet = {
        let cb = props.on_open_character_sheet.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };

    let on_travel = {
        let cb = props.on_travel_dying_forest.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };

    let on_inventory = {
        let cb = props.on_open_inventory.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };

    html! {
        <div class="screen town-screen">
            <div class="area-header">
                <div class="area-name">{ "Town Hub" }</div>
            </div>

            <div class="area-body">
                <div class="area-description">
                    <p>{ "Welcome to the Town. A moment of peace between battles." }</p>
                </div>

                <div class="town-actions">
                    {
                        if props.has_auto_combat {
                            html! {
                                <>
                                    <button class="btn btn-secondary town-btn" onclick={on_character_sheet}>
                                        { "Character Sheet" }
                                    </button>
                                    <button class="btn btn-secondary town-btn" onclick={on_inventory}>
                                        { "Inventory" }
                                    </button>
                                    <button class="btn btn-warning town-btn" onclick={on_travel}>
                                        { "Travel to The Dying Forest" }
                                    </button>
                                </>
                            }
                        } else {
                            html! {
                                <p>{ "This area is currently under construction." }</p>
                            }
                        }
                    }
                </div>
            </div>

            <div class="action-bar" style="justify-content: center;">
                <div class="action-buttons">
                    <button class="btn btn-danger" onclick={on_exit}>
                        { "Exit and Save" }
                    </button>
                </div>
            </div>
        </div>
    }
}
