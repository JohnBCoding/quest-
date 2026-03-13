use yew::prelude::*;

use quest_core::equipment::{EquipmentItem, EquipmentSlot};
use quest_core::item::{Item, ItemType};
use quest_core::player::Player;

#[derive(Properties, PartialEq)]
pub struct InventoryScreenProps {
    pub player: Player,
    pub equipped_main_hand: Option<EquipmentItem>,
    pub equipped_off_hand: Option<EquipmentItem>,
    pub equipped_head: Option<EquipmentItem>,
    pub equipped_body: Option<EquipmentItem>,
    pub equipped_hands: Option<EquipmentItem>,
    pub equipped_feet: Option<EquipmentItem>,
    pub inventory_items: Vec<EquipmentItem>,
    pub on_equip_main: Callback<String>,
    pub on_equip_off: Callback<String>,
    pub on_equip_head: Callback<String>,
    pub on_equip_body: Callback<String>,
    pub on_equip_hands: Callback<String>,
    pub on_equip_feet: Callback<String>,
    pub on_unequip_main: Callback<()>,
    pub on_unequip_off: Callback<()>,
    pub on_unequip_head: Callback<()>,
    pub on_unequip_body: Callback<()>,
    pub on_unequip_hands: Callback<()>,
    pub on_unequip_feet: Callback<()>,
    pub on_eat_fruit: Callback<String>,
    pub on_close: Callback<()>,
}

fn damage_range_label(item: &EquipmentItem) -> String {
    format!("{}-{} DMG", item.min_damage, item.max_damage)
}

fn render_equipped_slot(
    title: &str,
    item: &Option<EquipmentItem>,
    on_unequip: Callback<MouseEvent>,
) -> Html {
    html! {
        <div class="equipped-slot-card">
            <h3>{title}</h3>
            {
                if let Some(item) = item {
                    let detail = if item.max_damage > 0 {
                        format!("{} • Weight {}", damage_range_label(item), item.weight)
                    } else {
                        format!("Weight {}", item.weight)
                    };
                    html! {
                        <>
                            <div class="equipment-name">{&item.name}</div>
                            <div class="equipment-meta">{detail}</div>
                            <p class="equipment-description">{&item.description}</p>
                            <button class="btn btn-secondary" onclick={on_unequip}>{"Unequip"}</button>
                        </>
                    }
                } else {
                    html! {
                        <>
                            <div class="equipment-empty">{"Empty"}</div>
                            <p class="equipment-description">{"No item equipped in this slot."}</p>
                        </>
                    }
                }
            }
        </div>
    }
}

fn first_armor_slot(item: &EquipmentItem) -> Option<EquipmentSlot> {
    if item.can_equip_in(EquipmentSlot::Head) {
        Some(EquipmentSlot::Head)
    } else if item.can_equip_in(EquipmentSlot::Body) {
        Some(EquipmentSlot::Body)
    } else if item.can_equip_in(EquipmentSlot::Hands) {
        Some(EquipmentSlot::Hands)
    } else if item.can_equip_in(EquipmentSlot::Feet) {
        Some(EquipmentSlot::Feet)
    } else {
        None
    }
}

#[function_component(InventoryScreen)]
pub fn inventory_screen(props: &InventoryScreenProps) -> Html {
    let on_close = {
        let cb = props.on_close.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };

    let on_unequip_main = {
        let cb = props.on_unequip_main.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };
    let on_unequip_off = {
        let cb = props.on_unequip_off.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };
    let on_unequip_head = {
        let cb = props.on_unequip_head.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };
    let on_unequip_body = {
        let cb = props.on_unequip_body.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };
    let on_unequip_hands = {
        let cb = props.on_unequip_hands.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };
    let on_unequip_feet = {
        let cb = props.on_unequip_feet.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };

    let mut carried_fruits: Vec<Item> = props
        .player
        .list_item_inventory_items()
        .into_iter()
        .filter(|item| item.item_type == ItemType::Fruit)
        .collect();
    carried_fruits.sort_by(|a, b| a.name.cmp(&b.name));

    html! {
        <div class="screen inventory-screen">
            <div class="inventory-header">
                <h1>{"Inventory"}</h1>
                <div class="player-stat">{format!("{} - Lv. {}", props.player.name, props.player.level)}</div>
            </div>

            <div class="inventory-body">
                <section class="inventory-section">
                    <h2>{"Equipped"}</h2>
                    <div class="equipped-grid">
                        {render_equipped_slot("Main Hand", &props.equipped_main_hand, on_unequip_main)}
                        {render_equipped_slot("Off Hand", &props.equipped_off_hand, on_unequip_off)}
                        {render_equipped_slot("Head", &props.equipped_head, on_unequip_head)}
                        {render_equipped_slot("Body", &props.equipped_body, on_unequip_body)}
                        {render_equipped_slot("Hands", &props.equipped_hands, on_unequip_hands)}
                        {render_equipped_slot("Feet", &props.equipped_feet, on_unequip_feet)}
                    </div>
                </section>

                <section class="inventory-section">
                    <h2>{"Equipment"}</h2>
                    <div class="equipment-list">
                        {
                            if props.inventory_items.is_empty() {
                                html! {
                                    <div class="equipment-item-card empty">
                                        <div class="equipment-empty">{"No equipment in inventory"}</div>
                                    </div>
                                }
                            } else {
                                html! {
                                    {for props.inventory_items.iter().map(|item| {
                                        let is_two_handed = item.is_two_handed_weapon();
                                        let is_weapon = item.max_damage > 0;
                                        let can_main = item.can_equip_in(EquipmentSlot::MainHand);
                                        let can_off = item.can_equip_in(EquipmentSlot::OffHand);

                                        let equip_controls = if is_two_handed && is_weapon {
                                            let item_id = item.id.clone();
                                            let on_equip = {
                                                let cb = props.on_equip_main.clone();
                                                Callback::from(move |_: MouseEvent| cb.emit(item_id.clone()))
                                            };
                                            html! {
                                                <button class="btn btn-secondary" onclick={on_equip}>{"Equip (2H)"}</button>
                                            }
                                        } else if is_weapon && (can_main || can_off) {
                                            let item_id_for_main = item.id.clone();
                                            let on_equip_main = {
                                                let cb = props.on_equip_main.clone();
                                                Callback::from(move |_: MouseEvent| cb.emit(item_id_for_main.clone()))
                                            };

                                            let item_id_for_off = item.id.clone();
                                            let on_equip_off = {
                                                let cb = props.on_equip_off.clone();
                                                Callback::from(move |_: MouseEvent| cb.emit(item_id_for_off.clone()))
                                            };

                                            html! {
                                                <>
                                                    <button class="btn btn-secondary" onclick={on_equip_main} disabled={!can_main}>{"Equip Main"}</button>
                                                    <button class="btn btn-secondary" onclick={on_equip_off} disabled={!can_off}>{"Equip Off"}</button>
                                                </>
                                            }
                                        } else if let Some(slot) = first_armor_slot(item) {
                                            let item_id = item.id.clone();
                                            let on_equip = match slot {
                                                EquipmentSlot::Head => {
                                                    let cb = props.on_equip_head.clone();
                                                    Callback::from(move |_: MouseEvent| cb.emit(item_id.clone()))
                                                }
                                                EquipmentSlot::Body => {
                                                    let cb = props.on_equip_body.clone();
                                                    Callback::from(move |_: MouseEvent| cb.emit(item_id.clone()))
                                                }
                                                EquipmentSlot::Hands => {
                                                    let cb = props.on_equip_hands.clone();
                                                    Callback::from(move |_: MouseEvent| cb.emit(item_id.clone()))
                                                }
                                                EquipmentSlot::Feet => {
                                                    let cb = props.on_equip_feet.clone();
                                                    Callback::from(move |_: MouseEvent| cb.emit(item_id.clone()))
                                                }
                                                _ => Callback::from(|_: MouseEvent| {}),
                                            };

                                            html! {
                                                <button class="btn btn-secondary" onclick={on_equip}>{"Equip"}</button>
                                            }
                                        } else {
                                            html! {}
                                        };

                                        let detail = if item.max_damage > 0 {
                                            format!("{} • Weight {}", damage_range_label(item), item.weight)
                                        } else {
                                            format!("Weight {}", item.weight)
                                        };

                                        html! {
                                            <div class="equipment-item-card">
                                                <div class="equipment-name">{&item.name}</div>
                                                <div class="equipment-meta">{detail}</div>
                                                <p class="equipment-description">{&item.description}</p>
                                                <div class="equipment-actions">
                                                    { equip_controls }
                                                </div>
                                            </div>
                                        }
                                    })}
                                }
                            }
                        }
                    </div>
                </section>

                <section class="inventory-section">
                    <h2>{"Consumables"}</h2>
                    <div class="consumables-grid">
                        <div class="consumable-card">
                            <div class="consumable-title">{"Health Potion"}</div>
                            <div class="consumable-value">
                                {format!("Uses {}/{}", props.player.health_potion_uses, props.player.health_potion_capacity)}
                            </div>
                        </div>
                        <div class="consumable-card">
                            <div class="consumable-title">{"Fruits"}</div>
                            {
                                if carried_fruits.is_empty() {
                                    html! { <div class="consumable-empty">{"None"}</div> }
                                } else {
                                    html! {
                                        <div class="consumable-fruit-list">
                                            {for carried_fruits.iter().map(|fruit| {
                                                let fruit_id = fruit.id.clone();
                                                let on_eat = {
                                                    let cb = props.on_eat_fruit.clone();
                                                    Callback::from(move |_: MouseEvent| cb.emit(fruit_id.clone()))
                                                };
                                                html! {
                                                    <div class="consumable-fruit-row">
                                                        <div class="consumable-fruit-name">{&fruit.name}</div>
                                                        <div class="consumable-fruit-desc">
                                                            {fruit.effect.clone().unwrap_or_default()}
                                                        </div>
                                                        <button class="btn btn-secondary" onclick={on_eat}>{"Eat"}</button>
                                                    </div>
                                                }
                                            })}
                                        </div>
                                    }
                                }
                            }
                        </div>
                    </div>
                </section>
            </div>

            <div class="inventory-footer">
                <button class="btn btn-primary" onclick={on_close}>{"Close Inventory"}</button>
            </div>
        </div>
    }
}
