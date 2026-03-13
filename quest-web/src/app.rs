use yew::prelude::*;

use crate::screens::area::AreaScreen;
use crate::screens::character_sheet::CharacterSheetScreen;
use crate::screens::equipment_scene::EquipmentSceneScreen;
use crate::screens::fruit_scene::FruitSceneScreen;
use crate::screens::inventory::InventoryScreen;
use crate::screens::main_menu::MainMenuScreen;
use crate::screens::town::TownScreen;
use crate::storage;

use quest_core::action::Action;
use quest_core::equipment::EquipmentSlot;
use quest_core::game_state::ExecutedPlayerAction;
use quest_core::game_state::GameState;
use quest_core::item::{Item, ItemRarity, ItemType};
use quest_core::rng::RngManager;

#[derive(Clone, PartialEq)]
pub enum Screen {
    MainMenu,
    InGame,
    FruitScene,
    EquipmentScene,
    Inventory,
    CharacterSheet,
}

#[derive(Clone, PartialEq)]
pub enum TransitionState {
    None,
    WipeOut,
    WipeIn,
}

#[derive(Clone, Copy, PartialEq)]
pub enum TransitionEffect {
    Wipe,
    TownPortal,
}

#[derive(Clone, PartialEq)]
pub enum PostTransitionLogic {
    AdvanceEncounter,
    EatFruit,
    CompleteEquipmentScene,
    CloseInventory,
    CloseCharacterSheet,
    TravelToArea(String),
    PortalToTown,
}

#[derive(Clone, PartialEq)]
pub enum PlayerActionKind {
    Attack,
    Assassination,
    HealPotion,
}

#[derive(Clone, PartialEq)]
struct ItemDropPopup {
    id: u64,
    item_name: String,
    item_type_label: String,
    item_rarity: ItemRarity,
    is_entering: bool,
    is_exiting: bool,
    shift_count: u64,
}

pub struct App {
    screen: Screen,
    pending_screen: Option<Screen>,
    transition: TransitionState,
    transition_effect: TransitionEffect,
    game_state: Option<GameState>,
    rng_manager: Option<RngManager>,
    from_fruit_scene: bool,
    post_transition_logic: Option<PostTransitionLogic>,
    last_player_action_kind: Option<PlayerActionKind>,
    last_player_action_event_id: u64,
    last_mob_action_event_id: u64,
    area_combat_timer_epoch: u64,
    pending_portal_to_town: bool,
    action_progress_reset_event_id: u64,
    is_portal_to_town_transitioning: bool,
    item_drop_popups: Vec<ItemDropPopup>,
    next_item_drop_popup_id: u64,
    item_drop_timeout_token: u64,
    item_drop_timeout_scheduled: bool,
    suppress_next_drop_popup_enqueue: bool,
}

pub enum AppMsg {
    Navigate(Screen),
    TransitionMidpoint,
    TransitionEnd,
    NewGame,
    LoadGame,
    ExitGame,
    AttackMob,
    PerformAutoAction,
    QueuePortalToTown,
    MobAttack,
    AdvanceEncounter,
    AdvanceEncounterIfCurrent(u64),
    EnterPortal,
    EatFruit,
    EquipSceneItem,
    CloseCharacterSheet,
    OpenCharacterSheet,
    OpenInventory,
    CloseInventory,
    EquipMainHand(String),
    EquipOffHand(String),
    EquipHead(String),
    EquipBody(String),
    EquipHands(String),
    EquipFeet(String),
    UnequipMainHand,
    UnequipOffHand,
    UnequipHead,
    UnequipBody,
    UnequipHands,
    UnequipFeet,
    EatInventoryFruit(String),
    SaveActionPriority(Vec<Action>),
    TravelToArea(String),
    NavigateWithLogic(Screen, PostTransitionLogic),
    #[allow(dead_code)]
    EnqueueDroppedItems(Vec<(String, String, ItemRarity)>),
    PushDroppedItemPopup((String, String, ItemRarity)),
    FinalizeDroppedItemPopupEntry(u64),
    FinalizeDroppedItemPopupExit(u64),
    DroppedItemPopupTimeout(u64),
}

impl App {
    const ITEM_DROP_STAGGER_MS: u32 = 200;
    const ITEM_DROP_FIRST_TIMEOUT_MS: u32 = 2_000;
    const ITEM_DROP_CHAIN_TIMEOUT_MS: u32 = 250;
    const ITEM_DROP_ENTRY_ANIMATION_MS: u32 = 320;
    const ITEM_DROP_EXIT_ANIMATION_MS: u32 = 220;

    fn transition_out_duration_ms(effect: TransitionEffect) -> u32 {
        match effect {
            TransitionEffect::Wipe => 400,
            TransitionEffect::TownPortal => 700,
        }
    }

    fn transition_in_duration_ms(effect: TransitionEffect) -> u32 {
        match effect {
            TransitionEffect::Wipe => 400,
            TransitionEffect::TownPortal => 1063,
        }
    }

    fn start_transition(&mut self, ctx: &Context<Self>, effect: TransitionEffect) {
        self.transition_effect = effect;
        self.transition = TransitionState::WipeOut;
        let duration = Self::transition_out_duration_ms(effect);
        let link = ctx.link().clone();
        gloo_timers::callback::Timeout::new(duration, move || {
            link.send_message(AppMsg::TransitionMidpoint);
        })
        .forget();
    }

    fn transition_effect_for_logic(&self, logic: &PostTransitionLogic) -> TransitionEffect {
        match logic {
            PostTransitionLogic::PortalToTown => TransitionEffect::TownPortal,
            PostTransitionLogic::AdvanceEncounter => {
                if let Some(state) = self.game_state.as_ref() {
                    let is_non_beach_boss =
                        state.is_boss_encounter && state.current_area.id != "the_beach";
                    if is_non_beach_boss && state.portals_unlocked {
                        TransitionEffect::TownPortal
                    } else {
                        TransitionEffect::Wipe
                    }
                } else {
                    TransitionEffect::Wipe
                }
            }
            _ => TransitionEffect::Wipe,
        }
    }

    fn reset_area_combat_visual_state(&mut self) {
        self.last_player_action_kind = None;
        self.last_player_action_event_id = 0;
        self.last_mob_action_event_id = 0;
        self.action_progress_reset_event_id = self.action_progress_reset_event_id.saturating_add(1);
    }

    fn invalidate_area_combat_timers(&mut self) {
        self.area_combat_timer_epoch = self.area_combat_timer_epoch.saturating_add(1);
    }

    fn reset_item_drop_popups(&mut self) {
        self.item_drop_popups.clear();
        self.item_drop_timeout_scheduled = false;
        self.item_drop_timeout_token = self.item_drop_timeout_token.saturating_add(1);
        self.suppress_next_drop_popup_enqueue = false;
    }

    fn schedule_item_drop_timeout(&mut self, ctx: &Context<Self>, delay_ms: u32) {
        self.item_drop_timeout_token = self.item_drop_timeout_token.saturating_add(1);
        let timeout_token = self.item_drop_timeout_token;
        self.item_drop_timeout_scheduled = true;
        let link = ctx.link().clone();
        gloo_timers::callback::Timeout::new(delay_ms, move || {
            link.send_message(AppMsg::DroppedItemPopupTimeout(timeout_token));
        })
        .forget();
    }

    fn enqueue_dropped_item_names(
        &mut self,
        ctx: &Context<Self>,
        item_names: Vec<(String, String, ItemRarity)>,
    ) -> bool {
        if item_names.is_empty() {
            return false;
        }

        for (index, item_name) in item_names.into_iter().enumerate() {
            let delay_ms = (index as u32).saturating_mul(Self::ITEM_DROP_STAGGER_MS);
            let link = ctx.link().clone();
            gloo_timers::callback::Timeout::new(delay_ms, move || {
                link.send_message(AppMsg::PushDroppedItemPopup(item_name));
            })
            .forget();
        }

        false
    }

    fn item_type_label(item_type: ItemType) -> String {
        match item_type {
            ItemType::OneHandedSword => "One-Handed Sword".to_string(),
            ItemType::OneHandedDagger => "One-Handed Dagger".to_string(),
            ItemType::TwoHandedSword => "Two-Handed Sword".to_string(),
            ItemType::Helmet => "Helmet".to_string(),
            ItemType::BodyArmor => "Body Armor".to_string(),
            ItemType::Gloves => "Gloves".to_string(),
            ItemType::Boots => "Boots".to_string(),
            ItemType::Fruit => "Fruit".to_string(),
            ItemType::Unknown => "Item".to_string(),
        }
    }

    fn dropped_item_names_from_ids(item_ids: Vec<String>) -> Vec<(String, String, ItemRarity)> {
        item_ids
            .into_iter()
            .filter_map(|item_id| {
                Item::get_by_id(&item_id).map(|item| {
                    (
                        item.name,
                        Self::item_type_label(item.item_type),
                        item.rarity,
                    )
                })
            })
            .collect()
    }

    fn item_drop_display_duration_ms(item_count: usize) -> u32 {
        if item_count == 0 {
            return 0;
        }

        let item_count_u32 = item_count as u32;
        let chained_popups = (item_count.saturating_sub(1)) as u32;
        Self::ITEM_DROP_FIRST_TIMEOUT_MS
            .saturating_add(Self::ITEM_DROP_CHAIN_TIMEOUT_MS.saturating_mul(chained_popups))
            .saturating_add(Self::ITEM_DROP_EXIT_ANIMATION_MS.saturating_mul(item_count_u32))
            .saturating_add(120)
    }

    fn preview_boss_drop_items(&self) -> Vec<(String, String, ItemRarity)> {
        let Some(state) = self.game_state.as_ref() else {
            return Vec::new();
        };
        let Some(rng_manager) = self.rng_manager.as_ref() else {
            return Vec::new();
        };
        if !state.is_boss_encounter {
            return Vec::new();
        }
        let Some(mob) = state.current_mob.as_ref() else {
            return Vec::new();
        };
        if !mob.is_dead() {
            return Vec::new();
        }

        let mut simulated_state = state.clone();
        let mut simulated_rng = rng_manager.clone();
        let _ = simulated_state.advance_encounter_with_rng(&mut simulated_rng);
        Self::dropped_item_names_from_ids(simulated_state.take_recent_item_drop_ids())
    }

    fn schedule_advance_encounter_after_death(
        &mut self,
        ctx: &Context<Self>,
        is_dead: bool,
        is_boss_encounter: bool,
    ) {
        if !is_dead {
            return;
        }

        let epoch = self.area_combat_timer_epoch;
        let mut delay_ms = 2000;

        if is_boss_encounter {
            let preview_items = self.preview_boss_drop_items();
            if preview_items.is_empty() {
                delay_ms = 0;
            } else {
                let preview_count = preview_items.len();
                self.suppress_next_drop_popup_enqueue = true;
                ctx.link()
                    .send_message(AppMsg::EnqueueDroppedItems(preview_items));
                delay_ms = Self::item_drop_display_duration_ms(preview_count);
            }
        }

        if delay_ms == 0 {
            ctx.link()
                .send_message(AppMsg::AdvanceEncounterIfCurrent(epoch));
            return;
        }

        let link = ctx.link().clone();
        gloo_timers::callback::Timeout::new(delay_ms, move || {
            link.send_message(AppMsg::AdvanceEncounterIfCurrent(epoch));
        })
        .forget();
    }
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            screen: Screen::MainMenu,
            pending_screen: None,
            transition: TransitionState::None,
            transition_effect: TransitionEffect::Wipe,
            game_state: None,
            rng_manager: None,
            from_fruit_scene: false,
            post_transition_logic: None,
            last_player_action_kind: None,
            last_player_action_event_id: 0,
            last_mob_action_event_id: 0,
            area_combat_timer_epoch: 0,
            pending_portal_to_town: false,
            action_progress_reset_event_id: 0,
            is_portal_to_town_transitioning: false,
            item_drop_popups: Vec::new(),
            next_item_drop_popup_id: 1,
            item_drop_timeout_token: 0,
            item_drop_timeout_scheduled: false,
            suppress_next_drop_popup_enqueue: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMsg::Navigate(screen) => {
                self.pending_screen = Some(screen);
                self.start_transition(ctx, TransitionEffect::Wipe);
                true
            }
            AppMsg::NavigateWithLogic(screen, logic) => {
                let transition_effect = self.transition_effect_for_logic(&logic);
                self.pending_screen = Some(screen);
                self.post_transition_logic = Some(logic);
                self.start_transition(ctx, transition_effect);
                true
            }
            AppMsg::TransitionMidpoint => {
                if let Some(logic) = self.post_transition_logic.take() {
                    match logic {
                        PostTransitionLogic::AdvanceEncounter => {
                            let mut did_advance = false;
                            let mut fruit_scene_active = false;
                            let mut equipment_scene_active = false;
                            let mut entered_town = false;
                            let mut dropped_item_names: Vec<(String, String, ItemRarity)> =
                                Vec::new();
                            if let Some(ref mut state) = self.game_state {
                                let advanced = if let Some(mut rng) = self.rng_manager.take() {
                                    let result = state.advance_encounter_with_rng(&mut rng);
                                    dropped_item_names = Self::dropped_item_names_from_ids(
                                        state.take_recent_item_drop_ids(),
                                    );
                                    self.rng_manager = Some(rng);
                                    result
                                } else {
                                    state.advance_encounter()
                                };

                                if advanced {
                                    storage::save_game(state);
                                    did_advance = true;
                                    fruit_scene_active = state.fruit_scene_active;
                                    equipment_scene_active = state.equipment_scene_active;
                                    entered_town = state.in_town;
                                }
                            }
                            if self.suppress_next_drop_popup_enqueue {
                                self.suppress_next_drop_popup_enqueue = false;
                            } else if !dropped_item_names.is_empty() {
                                ctx.link()
                                    .send_message(AppMsg::EnqueueDroppedItems(dropped_item_names));
                            }
                            if did_advance {
                                if equipment_scene_active {
                                    self.invalidate_area_combat_timers();
                                    self.pending_portal_to_town = false;
                                    self.screen = Screen::EquipmentScene;
                                } else if fruit_scene_active {
                                    self.invalidate_area_combat_timers();
                                    self.pending_portal_to_town = false;
                                    self.screen = Screen::FruitScene;
                                } else {
                                    if entered_town {
                                        self.invalidate_area_combat_timers();
                                        self.pending_portal_to_town = false;
                                    }
                                    self.screen = Screen::InGame;
                                }
                            }
                        }
                        PostTransitionLogic::EatFruit => {
                            let mut ate_fruit = false;
                            if let Some(ref mut state) = self.game_state {
                                state.complete_fruit_scene();
                                storage::save_game(state);
                                ate_fruit = true;
                            }
                            if ate_fruit {
                                self.invalidate_area_combat_timers();
                                self.from_fruit_scene = true;
                                self.pending_portal_to_town = false;
                                self.screen = Screen::CharacterSheet;
                            }
                        }
                        PostTransitionLogic::CompleteEquipmentScene => {
                            if let Some(ref mut state) = self.game_state {
                                state.complete_equipment_scene();
                                storage::save_game(state);
                            }
                            self.pending_portal_to_town = false;
                            self.screen = Screen::Inventory;
                        }
                        PostTransitionLogic::CloseInventory => {
                            let mut entered_town = false;
                            if let Some(ref mut state) = self.game_state {
                                entered_town = state.finish_first_inventory_visit();
                                storage::save_game(state);
                            }
                            if entered_town {
                                self.invalidate_area_combat_timers();
                                self.pending_portal_to_town = false;
                            }
                            self.screen = Screen::InGame;
                        }
                        PostTransitionLogic::CloseCharacterSheet => {
                            if self.from_fruit_scene {
                                self.from_fruit_scene = false;
                                if let Some(ref mut state) = self.game_state {
                                    if let Some(mut rng) = self.rng_manager.take() {
                                        state.enter_area_with_rng("the_fringe", &mut rng);
                                        self.rng_manager = Some(rng);
                                    } else {
                                        state.enter_area("the_fringe");
                                    }
                                    storage::save_game(state);
                                }
                                self.reset_area_combat_visual_state();
                            }
                            self.pending_portal_to_town = false;
                            self.screen = Screen::InGame;
                        }
                        PostTransitionLogic::TravelToArea(area_id) => {
                            let mut entered_area = false;
                            if let Some(ref mut state) = self.game_state {
                                let entered = if let Some(mut rng) = self.rng_manager.take() {
                                    let result = state.enter_area_with_rng(&area_id, &mut rng);
                                    self.rng_manager = Some(rng);
                                    result
                                } else {
                                    state.enter_area(&area_id)
                                };
                                if entered {
                                    storage::save_game(state);
                                    entered_area = true;
                                }
                            }
                            if entered_area {
                                self.reset_area_combat_visual_state();
                                self.pending_portal_to_town = false;
                                self.screen = Screen::InGame;
                            }
                        }
                        PostTransitionLogic::PortalToTown => {
                            let mut entered_town = false;
                            if let Some(ref mut state) = self.game_state {
                                if state.portal_to_town() {
                                    storage::save_game(state);
                                    entered_town = true;
                                }
                            }
                            if entered_town {
                                self.invalidate_area_combat_timers();
                                self.pending_portal_to_town = false;
                                self.is_portal_to_town_transitioning = false;
                                self.screen = Screen::InGame;
                            }
                        }
                    }
                    self.pending_screen = None;
                } else if let Some(screen) = self.pending_screen.take() {
                    self.screen = screen;
                }
                self.transition = TransitionState::WipeIn;

                let link = ctx.link().clone();
                let duration = Self::transition_in_duration_ms(self.transition_effect);
                gloo_timers::callback::Timeout::new(duration, move || {
                    link.send_message(AppMsg::TransitionEnd);
                })
                .forget();

                true
            }
            AppMsg::TransitionEnd => {
                self.transition = TransitionState::None;
                self.transition_effect = TransitionEffect::Wipe;
                self.is_portal_to_town_transitioning = false;
                true
            }
            AppMsg::NewGame => {
                let (state, rng) = GameState::new_game();
                storage::save_game(&state);
                self.game_state = Some(state);
                self.rng_manager = Some(rng);
                self.reset_item_drop_popups();
                self.pending_portal_to_town = false;
                self.action_progress_reset_event_id = 0;
                self.is_portal_to_town_transitioning = false;
                self.last_player_action_kind = None;
                self.last_player_action_event_id = 0;
                self.last_mob_action_event_id = 0;
                self.area_combat_timer_epoch = 0;
                ctx.link().send_message(AppMsg::Navigate(Screen::InGame));
                false
            }
            AppMsg::LoadGame => {
                if let Some(state) = storage::load_game() {
                    let rng = state.restore_rng();
                    self.game_state = Some(state);
                    self.rng_manager = Some(rng);
                    self.reset_item_drop_popups();
                    self.pending_portal_to_town = false;
                    self.action_progress_reset_event_id = 0;
                    self.is_portal_to_town_transitioning = false;
                    self.last_player_action_kind = None;
                    self.last_player_action_event_id = 0;
                    self.last_mob_action_event_id = 0;
                    self.area_combat_timer_epoch = 0;
                    ctx.link().send_message(AppMsg::Navigate(Screen::InGame));
                }
                false
            }
            AppMsg::ExitGame => {
                self.invalidate_area_combat_timers();
                if let Some(ref state) = self.game_state {
                    storage::save_game(state);
                }
                self.game_state = None;
                self.rng_manager = None;
                self.reset_item_drop_popups();
                self.pending_portal_to_town = false;
                self.action_progress_reset_event_id = 0;
                self.is_portal_to_town_transitioning = false;
                self.last_player_action_kind = None;
                self.last_player_action_event_id = 0;
                self.last_mob_action_event_id = 0;
                ctx.link().send_message(AppMsg::Navigate(Screen::MainMenu));
                false
            }
            AppMsg::QueuePortalToTown => {
                if let Some(state) = self.game_state.as_ref() {
                    if !state.in_town && state.portals_unlocked && state.player.has_auto_combat() {
                        self.pending_portal_to_town = true;
                        self.action_progress_reset_event_id =
                            self.action_progress_reset_event_id.saturating_add(1);
                        return true;
                    }
                }
                false
            }
            AppMsg::AttackMob => {
                if let Some(ref mut state) = self.game_state {
                    let did_attack = if let Some(mut rng) = self.rng_manager.take() {
                        let attacked = state.execute_attack_with_rng(&mut rng);
                        self.rng_manager = Some(rng);
                        attacked
                    } else {
                        state.execute_attack()
                    };
                    if did_attack {
                        self.last_player_action_kind = Some(PlayerActionKind::Attack);
                        self.last_player_action_event_id =
                            self.last_player_action_event_id.saturating_add(1);
                        let is_dead = state.current_mob.as_ref().map_or(false, |m| m.is_dead());
                        let is_boss_encounter = state.is_boss_encounter;
                        storage::save_game(state);
                        self.schedule_advance_encounter_after_death(
                            ctx,
                            is_dead,
                            is_boss_encounter,
                        );

                        return true;
                    }
                }
                false
            }
            AppMsg::PerformAutoAction => {
                if let Some(ref mut state) = self.game_state {
                    let should_cancel_portal = state.in_town
                        || state.fruit_scene_active
                        || !state.player.is_alive()
                        || !state.portals_unlocked;
                    if self.pending_portal_to_town && should_cancel_portal {
                        self.pending_portal_to_town = false;
                        self.is_portal_to_town_transitioning = false;
                    }

                    if self.pending_portal_to_town {
                        self.pending_portal_to_town = false;
                        self.is_portal_to_town_transitioning = true;
                        ctx.link().send_message(AppMsg::NavigateWithLogic(
                            Screen::InGame,
                            PostTransitionLogic::PortalToTown,
                        ));
                        return true;
                    }

                    let executed = if let Some(mut rng) = self.rng_manager.take() {
                        let result = state.execute_prioritized_action_with_rng(&mut rng);
                        self.rng_manager = Some(rng);
                        result
                    } else {
                        state.execute_prioritized_action()
                    };

                    if let Some(executed) = executed {
                        let is_dead = state.current_mob.as_ref().map_or(false, |m| m.is_dead());
                        let is_boss_encounter = state.is_boss_encounter;
                        self.last_player_action_kind = match executed {
                            ExecutedPlayerAction::Attack => Some(PlayerActionKind::Attack),
                            ExecutedPlayerAction::Assassination => {
                                Some(PlayerActionKind::Assassination)
                            }
                            ExecutedPlayerAction::HealthPotion { .. } => {
                                Some(PlayerActionKind::HealPotion)
                            }
                        };
                        self.last_player_action_event_id =
                            self.last_player_action_event_id.saturating_add(1);
                        storage::save_game(state);
                        self.schedule_advance_encounter_after_death(
                            ctx,
                            is_dead,
                            is_boss_encounter,
                        );

                        return true;
                    }
                }
                false
            }
            AppMsg::MobAttack => {
                if self.is_portal_to_town_transitioning {
                    return false;
                }
                if let Some(ref mut state) = self.game_state {
                    let dealt = if let Some(mut rng) = self.rng_manager.take() {
                        let result = state.execute_mob_attack_with_rng(&mut rng);
                        self.rng_manager = Some(rng);
                        result
                    } else {
                        state.execute_mob_attack()
                    };
                    if dealt.is_some() {
                        if !state.player.is_alive() {
                            self.pending_portal_to_town = false;
                            self.is_portal_to_town_transitioning = false;
                        }
                        self.last_mob_action_event_id =
                            self.last_mob_action_event_id.saturating_add(1);
                        storage::save_game(state);
                        return true;
                    }
                }
                false
            }
            AppMsg::AdvanceEncounter => {
                self.pending_portal_to_town = false;
                self.is_portal_to_town_transitioning = false;
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
                    let mut dropped_item_names: Vec<(String, String, ItemRarity)> = Vec::new();
                    if let Some(ref mut state) = self.game_state {
                        let advanced = if let Some(mut rng) = self.rng_manager.take() {
                            let result = state.advance_encounter_with_rng(&mut rng);
                            dropped_item_names = Self::dropped_item_names_from_ids(
                                state.take_recent_item_drop_ids(),
                            );
                            self.rng_manager = Some(rng);
                            result
                        } else {
                            state.advance_encounter()
                        };
                        if advanced {
                            storage::save_game(state);
                        }
                    }
                    if self.suppress_next_drop_popup_enqueue {
                        self.suppress_next_drop_popup_enqueue = false;
                    } else if !dropped_item_names.is_empty() {
                        ctx.link()
                            .send_message(AppMsg::EnqueueDroppedItems(dropped_item_names));
                    }
                    true
                }
            }
            AppMsg::AdvanceEncounterIfCurrent(epoch) => {
                if epoch != self.area_combat_timer_epoch {
                    return false;
                }
                ctx.link().send_message(AppMsg::AdvanceEncounter);
                false
            }
            AppMsg::EnterPortal => {
                self.pending_portal_to_town = false;
                self.is_portal_to_town_transitioning = false;
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
            AppMsg::EquipSceneItem => {
                ctx.link().send_message(AppMsg::NavigateWithLogic(
                    Screen::Inventory,
                    PostTransitionLogic::CompleteEquipmentScene,
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
            AppMsg::OpenInventory => {
                ctx.link().send_message(AppMsg::Navigate(Screen::Inventory));
                false
            }
            AppMsg::CloseInventory => {
                ctx.link().send_message(AppMsg::NavigateWithLogic(
                    Screen::InGame,
                    PostTransitionLogic::CloseInventory,
                ));
                false
            }
            AppMsg::EquipMainHand(item_id) => {
                if let Some(ref mut state) = self.game_state {
                    if state
                        .player
                        .equip_item_to_slot(&item_id, EquipmentSlot::MainHand)
                    {
                        storage::save_game(state);
                        return true;
                    }
                }
                false
            }
            AppMsg::EquipOffHand(item_id) => {
                if let Some(ref mut state) = self.game_state {
                    if state
                        .player
                        .equip_item_to_slot(&item_id, EquipmentSlot::OffHand)
                    {
                        storage::save_game(state);
                        return true;
                    }
                }
                false
            }
            AppMsg::EquipHead(item_id) => {
                if let Some(ref mut state) = self.game_state {
                    if state
                        .player
                        .equip_item_to_slot(&item_id, EquipmentSlot::Head)
                    {
                        storage::save_game(state);
                        return true;
                    }
                }
                false
            }
            AppMsg::EquipBody(item_id) => {
                if let Some(ref mut state) = self.game_state {
                    if state
                        .player
                        .equip_item_to_slot(&item_id, EquipmentSlot::Body)
                    {
                        storage::save_game(state);
                        return true;
                    }
                }
                false
            }
            AppMsg::EquipHands(item_id) => {
                if let Some(ref mut state) = self.game_state {
                    if state
                        .player
                        .equip_item_to_slot(&item_id, EquipmentSlot::Hands)
                    {
                        storage::save_game(state);
                        return true;
                    }
                }
                false
            }
            AppMsg::EquipFeet(item_id) => {
                if let Some(ref mut state) = self.game_state {
                    if state
                        .player
                        .equip_item_to_slot(&item_id, EquipmentSlot::Feet)
                    {
                        storage::save_game(state);
                        return true;
                    }
                }
                false
            }
            AppMsg::UnequipMainHand => {
                if let Some(ref mut state) = self.game_state {
                    if state.player.unequip_slot(EquipmentSlot::MainHand) {
                        storage::save_game(state);
                        return true;
                    }
                }
                false
            }
            AppMsg::UnequipHead => {
                if let Some(ref mut state) = self.game_state {
                    if state.player.unequip_slot(EquipmentSlot::Head) {
                        storage::save_game(state);
                        return true;
                    }
                }
                false
            }
            AppMsg::UnequipBody => {
                if let Some(ref mut state) = self.game_state {
                    if state.player.unequip_slot(EquipmentSlot::Body) {
                        storage::save_game(state);
                        return true;
                    }
                }
                false
            }
            AppMsg::UnequipHands => {
                if let Some(ref mut state) = self.game_state {
                    if state.player.unequip_slot(EquipmentSlot::Hands) {
                        storage::save_game(state);
                        return true;
                    }
                }
                false
            }
            AppMsg::UnequipFeet => {
                if let Some(ref mut state) = self.game_state {
                    if state.player.unequip_slot(EquipmentSlot::Feet) {
                        storage::save_game(state);
                        return true;
                    }
                }
                false
            }
            AppMsg::EatInventoryFruit(item_id) => {
                if let Some(ref mut state) = self.game_state {
                    if state.consume_inventory_fruit(&item_id) {
                        storage::save_game(state);
                        return true;
                    }
                }
                false
            }
            AppMsg::UnequipOffHand => {
                if let Some(ref mut state) = self.game_state {
                    if state.player.unequip_slot(EquipmentSlot::OffHand) {
                        storage::save_game(state);
                        return true;
                    }
                }
                false
            }
            AppMsg::SaveActionPriority(actions) => {
                if let Some(ref mut state) = self.game_state {
                    state.player.actions = actions;
                    storage::save_game(state);
                    return true;
                }
                false
            }
            AppMsg::TravelToArea(area_id) => {
                ctx.link().send_message(AppMsg::NavigateWithLogic(
                    Screen::InGame,
                    PostTransitionLogic::TravelToArea(area_id),
                ));
                false
            }
            AppMsg::EnqueueDroppedItems(item_names) => {
                self.enqueue_dropped_item_names(ctx, item_names)
            }
            AppMsg::PushDroppedItemPopup((item_name, item_type_label, item_rarity)) => {
                let popup_id = self.next_item_drop_popup_id;
                self.next_item_drop_popup_id = self.next_item_drop_popup_id.saturating_add(1);
                self.item_drop_popups.push(ItemDropPopup {
                    id: popup_id,
                    item_name,
                    item_type_label,
                    item_rarity,
                    is_entering: true,
                    is_exiting: false,
                    shift_count: 0,
                });

                let link = ctx.link().clone();
                gloo_timers::callback::Timeout::new(
                    Self::ITEM_DROP_ENTRY_ANIMATION_MS,
                    move || {
                        link.send_message(AppMsg::FinalizeDroppedItemPopupEntry(popup_id));
                    },
                )
                .forget();

                if self.item_drop_popups.len() == 1 && !self.item_drop_timeout_scheduled {
                    self.schedule_item_drop_timeout(ctx, Self::ITEM_DROP_FIRST_TIMEOUT_MS);
                }

                true
            }
            AppMsg::FinalizeDroppedItemPopupEntry(popup_id) => {
                if let Some(popup) = self
                    .item_drop_popups
                    .iter_mut()
                    .find(|popup| popup.id == popup_id)
                {
                    if popup.is_entering {
                        popup.is_entering = false;
                        return true;
                    }
                }
                false
            }
            AppMsg::DroppedItemPopupTimeout(timeout_token) => {
                if timeout_token != self.item_drop_timeout_token
                    || !self.item_drop_timeout_scheduled
                {
                    return false;
                }

                self.item_drop_timeout_scheduled = false;

                if self.item_drop_popups.is_empty() {
                    return false;
                }

                if let Some(first_popup) = self.item_drop_popups.first_mut() {
                    if first_popup.is_exiting {
                        return false;
                    }

                    first_popup.is_exiting = true;
                    let popup_id = first_popup.id;
                    let link = ctx.link().clone();
                    gloo_timers::callback::Timeout::new(
                        Self::ITEM_DROP_EXIT_ANIMATION_MS,
                        move || {
                            link.send_message(AppMsg::FinalizeDroppedItemPopupExit(popup_id));
                        },
                    )
                    .forget();
                    return true;
                }

                false
            }
            AppMsg::FinalizeDroppedItemPopupExit(popup_id) => {
                let Some(removed_index) = self
                    .item_drop_popups
                    .iter()
                    .position(|popup| popup.id == popup_id)
                else {
                    return false;
                };
                self.item_drop_popups.remove(removed_index);

                if self.item_drop_popups.is_empty() {
                    return true;
                }

                for popup in &mut self.item_drop_popups {
                    popup.shift_count = popup.shift_count.saturating_add(1);
                }

                self.schedule_item_drop_timeout(ctx, Self::ITEM_DROP_CHAIN_TIMEOUT_MS);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let transition_class = match (self.transition.clone(), self.transition_effect) {
            (TransitionState::None, _) => "",
            (TransitionState::WipeOut, TransitionEffect::Wipe) => "transition-wipe-out",
            (TransitionState::WipeIn, TransitionEffect::Wipe) => "transition-wipe-in",
            (TransitionState::WipeOut, TransitionEffect::TownPortal) => {
                "transition-town-portal-out"
            }
            (TransitionState::WipeIn, TransitionEffect::TownPortal) => "transition-town-portal-in",
        };

        let item_drop_popups: Vec<Html> = self
            .item_drop_popups
            .iter()
            .map(|popup| {
                let rarity_class = match popup.item_rarity {
                    ItemRarity::Common => "item-drop-popup-common",
                    ItemRarity::Uncommon => "item-drop-popup-uncommon",
                    ItemRarity::Rare => "item-drop-popup-rare",
                };
                let enter_class = if popup.is_entering {
                    Some(match popup.item_rarity {
                        ItemRarity::Common => "item-drop-popup-enter-common",
                        ItemRarity::Uncommon => "item-drop-popup-enter-uncommon",
                        ItemRarity::Rare => "item-drop-popup-enter-rare",
                    })
                } else {
                    None
                };
                let popup_classes = classes!(
                    "item-drop-popup",
                    rarity_class,
                    enter_class,
                    popup.is_exiting.then_some("item-drop-popup-exit"),
                    (!popup.is_entering && !popup.is_exiting && popup.shift_count > 0)
                        .then_some("item-drop-popup-shift")
                );

                html! {
                    <div
                        key={format!("{}-{}", popup.id, popup.shift_count)}
                        class={popup_classes}
                    >
                        <div class="item-drop-popup-label">{ &popup.item_type_label }</div>
                        <div class="item-drop-popup-name">{ &popup.item_name }</div>
                    </div>
                }
            })
            .collect();

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
                let on_auto_action = ctx.link().callback(|_| AppMsg::PerformAutoAction);
                let on_mob_attack = ctx.link().callback(|_| AppMsg::MobAttack);
                let on_enter_portal = ctx.link().callback(|_| AppMsg::EnterPortal);
                let on_portal_to_town = ctx.link().callback(|_| AppMsg::QueuePortalToTown);
                if let Some(ref state) = self.game_state {
                    if state.in_town {
                        let on_open_cs = ctx.link().callback(|_| AppMsg::OpenCharacterSheet);
                        let on_open_inventory = ctx.link().callback(|_| AppMsg::OpenInventory);
                        let on_travel = ctx
                            .link()
                            .callback(|_| AppMsg::TravelToArea("dying_forest".to_string()));
                        html! {
                            <TownScreen
                                has_auto_combat={state.player.has_auto_combat()}
                                on_exit={on_exit}
                                on_open_character_sheet={on_open_cs}
                                on_open_inventory={on_open_inventory}
                                on_travel_dying_forest={on_travel}
                            />
                        }
                    } else {
                        html! {
                            <div class="area-screen-shell">
                                <AreaScreen
                                    area={state.current_area.clone()}
                                    player={state.player.clone()}
                                    current_mob={state.current_mob.clone()}
                                    encounters_cleared={state.encounters_cleared}
                                    is_boss={state.is_boss_encounter}
                                    has_auto_combat={state.player.has_auto_combat()}
                                    on_exit={on_exit}
                                    on_attack={on_attack}
                                    on_auto_action={on_auto_action}
                                    on_mob_attack={on_mob_attack}
                                    on_enter_portal={on_enter_portal}
                                    on_portal_to_town={on_portal_to_town}
                                    can_portal_to_town={state.portals_unlocked}
                                    is_portal_to_town_pending={self.pending_portal_to_town}
                                    action_progress_reset_event_id={self.action_progress_reset_event_id}
                                    is_portal_to_town_transitioning={self.is_portal_to_town_transitioning}
                                    last_player_action_kind={self.last_player_action_kind.clone()}
                                    player_action_event_id={self.last_player_action_event_id}
                                    mob_action_event_id={self.last_mob_action_event_id}
                                />
                                <div class="item-drop-popup-strip" aria-live="polite" aria-atomic="false">
                                    { for item_drop_popups.iter().cloned() }
                                </div>
                            </div>
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
            Screen::EquipmentScene => {
                if let Some(ref state) = self.game_state {
                    let item_id = state.pending_equipment_id.clone().unwrap_or_default();
                    let on_equip_item = ctx.link().callback(|_| AppMsg::EquipSceneItem);
                    html! {
                        <EquipmentSceneScreen
                            item_id={item_id}
                            on_equip_item={on_equip_item}
                        />
                    }
                } else {
                    html! { <div class="screen">{ "Error: No game state" }</div> }
                }
            }
            Screen::Inventory => {
                if let Some(ref state) = self.game_state {
                    let on_close = ctx.link().callback(|_| AppMsg::CloseInventory);
                    let on_equip_main = ctx.link().callback(AppMsg::EquipMainHand);
                    let on_equip_off = ctx.link().callback(AppMsg::EquipOffHand);
                    let on_equip_head = ctx.link().callback(AppMsg::EquipHead);
                    let on_equip_body = ctx.link().callback(AppMsg::EquipBody);
                    let on_equip_hands = ctx.link().callback(AppMsg::EquipHands);
                    let on_equip_feet = ctx.link().callback(AppMsg::EquipFeet);
                    let on_unequip_main = ctx.link().callback(|_| AppMsg::UnequipMainHand);
                    let on_unequip_off = ctx.link().callback(|_| AppMsg::UnequipOffHand);
                    let on_unequip_head = ctx.link().callback(|_| AppMsg::UnequipHead);
                    let on_unequip_body = ctx.link().callback(|_| AppMsg::UnequipBody);
                    let on_unequip_hands = ctx.link().callback(|_| AppMsg::UnequipHands);
                    let on_unequip_feet = ctx.link().callback(|_| AppMsg::UnequipFeet);
                    let on_eat_fruit = ctx.link().callback(AppMsg::EatInventoryFruit);
                    html! {
                        <InventoryScreen
                            player={state.player.clone()}
                            equipped_main_hand={state.player.equipped_item(EquipmentSlot::MainHand)}
                            equipped_off_hand={state.player.equipped_item(EquipmentSlot::OffHand)}
                            equipped_head={state.player.equipped_item(EquipmentSlot::Head)}
                            equipped_body={state.player.equipped_item(EquipmentSlot::Body)}
                            equipped_hands={state.player.equipped_item(EquipmentSlot::Hands)}
                            equipped_feet={state.player.equipped_item(EquipmentSlot::Feet)}
                            inventory_items={state.player.list_equipment_inventory_items()}
                            on_equip_main={on_equip_main}
                            on_equip_off={on_equip_off}
                            on_equip_head={on_equip_head}
                            on_equip_body={on_equip_body}
                            on_equip_hands={on_equip_hands}
                            on_equip_feet={on_equip_feet}
                            on_unequip_main={on_unequip_main}
                            on_unequip_off={on_unequip_off}
                            on_unequip_head={on_unequip_head}
                            on_unequip_body={on_unequip_body}
                            on_unequip_hands={on_unequip_hands}
                            on_unequip_feet={on_unequip_feet}
                            on_eat_fruit={on_eat_fruit}
                            on_close={on_close}
                        />
                    }
                } else {
                    html! { <div class="screen">{ "Error: No game state" }</div> }
                }
            }
            Screen::CharacterSheet => {
                if let Some(ref state) = self.game_state {
                    let on_close = ctx.link().callback(|_| AppMsg::CloseCharacterSheet);
                    let on_save_actions = ctx
                        .link()
                        .callback(|actions| AppMsg::SaveActionPriority(actions));
                    html! {
                        <CharacterSheetScreen
                            player={state.player.clone()}
                            on_close={on_close}
                            on_save_actions={on_save_actions}
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
