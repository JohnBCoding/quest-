use yew::prelude::*;

use crate::screens::area::AreaScreen;
use crate::screens::character_sheet::CharacterSheetScreen;
use crate::screens::fruit_scene::FruitSceneScreen;
use crate::screens::main_menu::MainMenuScreen;
use crate::screens::town::TownScreen;
use crate::storage;

use quest_core::game_state::GameState;
use quest_core::rng::RngManager;

#[derive(Clone, PartialEq)]
pub enum Screen {
    MainMenu,
    InGame,
    FruitScene,
    CharacterSheet,
}

#[derive(Clone, PartialEq)]
pub enum TransitionState {
    None,
    WipeOut,
    WipeIn,
}

#[derive(Clone, PartialEq)]
pub enum PostTransitionLogic {
    AdvanceEncounter,
    EatFruit,
    CloseCharacterSheet,
    TravelToArea(String),
}

pub struct App {
    screen: Screen,
    pending_screen: Option<Screen>,
    transition: TransitionState,
    game_state: Option<GameState>,
    rng_manager: Option<RngManager>,
    from_fruit_scene: bool,
    post_transition_logic: Option<PostTransitionLogic>,
}

pub enum AppMsg {
    Navigate(Screen),
    TransitionMidpoint,
    TransitionEnd,
    NewGame,
    LoadGame,
    ExitGame,
    AttackMob,
    MobAttack,
    AdvanceEncounter,
    EnterPortal,
    EatFruit,
    CloseCharacterSheet,
    OpenCharacterSheet,
    TravelToArea(String),
    NavigateWithLogic(Screen, PostTransitionLogic),
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
            from_fruit_scene: false,
            post_transition_logic: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMsg::Navigate(screen) => {
                self.pending_screen = Some(screen);
                self.transition = TransitionState::WipeOut;

                let link = ctx.link().clone();
                gloo_timers::callback::Timeout::new(400, move || {
                    link.send_message(AppMsg::TransitionMidpoint);
                })
                .forget();

                true
            }
            AppMsg::NavigateWithLogic(screen, logic) => {
                self.pending_screen = Some(screen);
                self.post_transition_logic = Some(logic);
                self.transition = TransitionState::WipeOut;

                let link = ctx.link().clone();
                gloo_timers::callback::Timeout::new(400, move || {
                    link.send_message(AppMsg::TransitionMidpoint);
                })
                .forget();

                true
            }
            AppMsg::TransitionMidpoint => {
                if let Some(logic) = self.post_transition_logic.take() {
                    match logic {
                        PostTransitionLogic::AdvanceEncounter => {
                            if let Some(ref mut state) = self.game_state {
                                if state.advance_encounter() {
                                    storage::save_game(state);
                                    if state.fruit_scene_active {
                                        self.screen = Screen::FruitScene;
                                    } else {
                                        self.screen = Screen::InGame;
                                    }
                                }
                            }
                        }
                        PostTransitionLogic::EatFruit => {
                            if let Some(ref mut state) = self.game_state {
                                state.complete_fruit_scene();
                                storage::save_game(state);
                                self.from_fruit_scene = true;
                                self.screen = Screen::CharacterSheet;
                            }
                        }
                        PostTransitionLogic::CloseCharacterSheet => {
                            if self.from_fruit_scene {
                                self.from_fruit_scene = false;
                                if let Some(ref mut state) = self.game_state {
                                    state.enter_area("the_fringe");
                                    storage::save_game(state);
                                }
                            }
                            self.screen = Screen::InGame;
                        }
                        PostTransitionLogic::TravelToArea(area_id) => {
                            if let Some(ref mut state) = self.game_state {
                                if state.enter_area(&area_id) {
                                    storage::save_game(state);
                                    self.screen = Screen::InGame;
                                }
                            }
                        }
                    }
                    self.pending_screen = None;
                } else if let Some(screen) = self.pending_screen.take() {
                    self.screen = screen;
                }
                self.transition = TransitionState::WipeIn;

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
                if let Some(ref state) = self.game_state {
                    storage::save_game(state);
                }
                self.game_state = None;
                self.rng_manager = None;
                ctx.link().send_message(AppMsg::Navigate(Screen::MainMenu));
                false
            }
            AppMsg::AttackMob => {
                if let Some(ref mut state) = self.game_state {
                    if state.execute_attack() {
                        let is_dead = state.current_mob.as_ref().map_or(false, |m| m.is_dead());
                        storage::save_game(state);

                        if is_dead {
                            let link = ctx.link().clone();
                            gloo_timers::callback::Timeout::new(2000, move || {
                                link.send_message(AppMsg::AdvanceEncounter);
                            })
                            .forget();
                        }

                        return true;
                    }
                }
                false
            }
            AppMsg::MobAttack => {
                if let Some(ref mut state) = self.game_state {
                    if state.execute_mob_attack().is_some() {
                        storage::save_game(state);
                        return true;
                    }
                }
                false
            }
            AppMsg::AdvanceEncounter => {
                let mut needs_wipe = false;

                if let Some(ref mut state) = self.game_state {
                    // Check if this move will result in a screen change
                    let is_beach_boss =
                        state.is_boss_encounter && state.current_area.id == "the_beach";
                    let is_other_boss =
                        state.is_boss_encounter && state.current_area.id != "the_beach";
                    let _is_last_encounter = !state.is_boss_encounter
                        && state.encounters_cleared + 1 >= state.current_area.base_encounter_amount;

                    // If boss death (beach -> fruit, other -> town) or area end (portal state), those are big shifts
                    // Actually, even portal state might be better without a wipe if we want the shimmer to show.
                    // The user said "boss portal ... already have animations".

                    if is_beach_boss || is_other_boss {
                        needs_wipe = true;
                    }
                }

                if needs_wipe {
                    ctx.link().send_message(AppMsg::NavigateWithLogic(
                        Screen::InGame,
                        PostTransitionLogic::AdvanceEncounter,
                    ));
                    false
                } else {
                    // Local change (regular mob or portal shimmer), handle instantly
                    if let Some(ref mut state) = self.game_state {
                        if state.advance_encounter() {
                            storage::save_game(state);
                        }
                    }
                    true
                }
            }
            AppMsg::EnterPortal => {
                let mut state_changed = false;
                if let Some(ref mut state) = self.game_state {
                    if let Some(mut rng) = self.rng_manager.take() {
                        if state.enter_boss_portal(&mut rng) {
                            storage::save_game(state);
                            state_changed = true;
                        }
                        self.rng_manager = Some(rng);
                    }
                }
                state_changed
            }
            AppMsg::EatFruit => {
                ctx.link().send_message(AppMsg::NavigateWithLogic(
                    Screen::CharacterSheet,
                    PostTransitionLogic::EatFruit,
                ));
                false
            }
            AppMsg::CloseCharacterSheet => {
                ctx.link().send_message(AppMsg::NavigateWithLogic(
                    Screen::InGame,
                    PostTransitionLogic::CloseCharacterSheet,
                ));
                false
            }
            AppMsg::OpenCharacterSheet => {
                ctx.link()
                    .send_message(AppMsg::Navigate(Screen::CharacterSheet));
                false
            }
            AppMsg::TravelToArea(area_id) => {
                ctx.link().send_message(AppMsg::NavigateWithLogic(
                    Screen::InGame,
                    PostTransitionLogic::TravelToArea(area_id),
                ));
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
                let on_attack = ctx.link().callback(|_| AppMsg::AttackMob);
                let on_mob_attack = ctx.link().callback(|_| AppMsg::MobAttack);
                let on_enter_portal = ctx.link().callback(|_| AppMsg::EnterPortal);
                if let Some(ref state) = self.game_state {
                    if state.in_town {
                        let on_open_cs = ctx.link().callback(|_| AppMsg::OpenCharacterSheet);
                        let on_travel = ctx
                            .link()
                            .callback(|_| AppMsg::TravelToArea("the_fringe".to_string()));
                        html! {
                            <TownScreen
                                has_auto_combat={state.player.has_auto_combat()}
                                on_exit={on_exit}
                                on_open_character_sheet={on_open_cs}
                                on_travel_fringe={on_travel}
                            />
                        }
                    } else {
                        html! {
                            <AreaScreen
                                area={state.current_area.clone()}
                                player={state.player.clone()}
                                current_mob={state.current_mob.clone()}
                                encounters_cleared={state.encounters_cleared}
                                is_boss={state.is_boss_encounter}
                                has_auto_combat={state.player.has_auto_combat()}
                                on_exit={on_exit}
                                on_attack={on_attack}
                                on_mob_attack={on_mob_attack}
                                on_enter_portal={on_enter_portal}
                            />
                        }
                    }
                } else {
                    html! { <div class="screen">{ "Error: No game state" }</div> }
                }
            }
            Screen::FruitScene => {
                if let Some(ref state) = self.game_state {
                    let fruit_id = state.pending_fruit_id.clone().unwrap_or_default();
                    let on_eat = ctx.link().callback(|_| AppMsg::EatFruit);
                    html! {
                        <FruitSceneScreen
                            fruit_id={fruit_id}
                            on_eat_fruit={on_eat}
                        />
                    }
                } else {
                    html! { <div class="screen">{ "Error: No game state" }</div> }
                }
            }
            Screen::CharacterSheet => {
                if let Some(ref state) = self.game_state {
                    let on_close = ctx.link().callback(|_| AppMsg::CloseCharacterSheet);
                    html! {
                        <CharacterSheetScreen
                            player={state.player.clone()}
                            on_close={on_close}
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
                <div class={classes!("transition-overlay", transition_class)} />
            </div>
        }
    }
}
