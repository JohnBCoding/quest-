use yew::prelude::*;

use crate::screens::area::AreaScreen;
use crate::screens::main_menu::MainMenuScreen;
use crate::storage;

use quest_core::game_state::GameState;
use quest_core::rng::RngManager;

/// The active screen in the app.
#[derive(Clone, PartialEq)]
pub enum Screen {
    MainMenu,
    InGame,
}

/// Transition state for old-school screen wipe.
#[derive(Clone, PartialEq)]
pub enum TransitionState {
    None,
    /// Wipe-out: covering the screen (old content fading)
    WipeOut,
    /// Wipe-in: revealing new screen
    WipeIn,
}

pub struct App {
    screen: Screen,
    pending_screen: Option<Screen>,
    transition: TransitionState,
    game_state: Option<GameState>,
    rng_manager: Option<RngManager>,
}

pub enum AppMsg {
    Navigate(Screen),
    TransitionMidpoint,
    TransitionEnd,
    NewGame,
    LoadGame,
    ExitGame,
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            screen: Screen::MainMenu,
            pending_screen: None,
            transition: TransitionState::None,
            game_state: None,
            rng_manager: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMsg::Navigate(screen) => {
                self.pending_screen = Some(screen);
                self.transition = TransitionState::WipeOut;

                // After wipe-out animation, trigger midpoint
                let link = ctx.link().clone();
                gloo_timers::callback::Timeout::new(400, move || {
                    link.send_message(AppMsg::TransitionMidpoint);
                })
                .forget();

                true
            }
            AppMsg::TransitionMidpoint => {
                // Swap the screen while fully covered
                if let Some(screen) = self.pending_screen.take() {
                    self.screen = screen;
                }
                self.transition = TransitionState::WipeIn;

                // After wipe-in, clear transition
                let link = ctx.link().clone();
                gloo_timers::callback::Timeout::new(400, move || {
                    link.send_message(AppMsg::TransitionEnd);
                })
                .forget();

                true
            }
            AppMsg::TransitionEnd => {
                self.transition = TransitionState::None;
                true
            }
            AppMsg::NewGame => {
                let (state, rng) = GameState::new_game();
                storage::save_game(&state);
                self.game_state = Some(state);
                self.rng_manager = Some(rng);
                ctx.link().send_message(AppMsg::Navigate(Screen::InGame));
                false
            }
            AppMsg::LoadGame => {
                if let Some(state) = storage::load_game() {
                    let rng = state.restore_rng();
                    self.game_state = Some(state);
                    self.rng_manager = Some(rng);
                    ctx.link().send_message(AppMsg::Navigate(Screen::InGame));
                }
                false
            }
            AppMsg::ExitGame => {
                // Save before exiting
                if let Some(ref state) = self.game_state {
                    storage::save_game(state);
                }
                self.game_state = None;
                self.rng_manager = None;
                ctx.link().send_message(AppMsg::Navigate(Screen::MainMenu));
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let transition_class = match self.transition {
            TransitionState::None => "",
            TransitionState::WipeOut => "transition-wipe-out",
            TransitionState::WipeIn => "transition-wipe-in",
        };

        let content = match self.screen {
            Screen::MainMenu => {
                let on_new_game = ctx.link().callback(|_| AppMsg::NewGame);
                let on_load_game = ctx.link().callback(|_| AppMsg::LoadGame);
                html! {
                    <MainMenuScreen
                        on_new_game={on_new_game}
                        on_load_game={on_load_game}
                    />
                }
            }
            Screen::InGame => {
                let on_exit = ctx.link().callback(|_| AppMsg::ExitGame);
                if let Some(ref state) = self.game_state {
                    html! {
                        <AreaScreen
                            area={state.current_area.clone()}
                            player={state.player.clone()}
                            on_exit={on_exit}
                        />
                    }
                } else {
                    html! { <div class="screen">{ "Error: No game state" }</div> }
                }
            }
        };

        html! {
            <div class="app">
                <div class={classes!("screen-container", transition_class)}>
                    { content }
                </div>
                // Transition overlay
                <div class={classes!("transition-overlay", transition_class)} />
            </div>
        }
    }
}
