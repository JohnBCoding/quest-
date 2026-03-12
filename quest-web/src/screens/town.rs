use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TownScreenProps {
    pub on_exit: Callback<()>,
}

#[function_component(TownScreen)]
pub fn town_screen(props: &TownScreenProps) -> Html {
    let on_exit = {
        let cb = props.on_exit.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };

    html! {
        <div class="screen town-screen">
            <div class="area-header">
                <div class="area-name">{ "Town Hub" }</div>
            </div>

            <div class="area-body">
                <div class="area-description">
                    <p>{ "Welcome to the Town. It's quiet here. Very quiet." }</p>
                    <p>{ "This area is currently under construction." }</p>
                </div>
            </div>

            <div class="action-bar" style="justify-content: center;">
                <div class="action-buttons">
                    <button class="btn btn-danger" onclick={on_exit}>
                        { "Exit Game" }
                    </button>
                </div>
            </div>
        </div>
    }
}
