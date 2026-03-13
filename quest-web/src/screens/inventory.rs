use yew::prelude::*;

use quest_core::equipment::{EquipmentItem, EquipmentSlot};
use quest_core::fruit::FRUIT_REGISTRY;
use quest_core::player::Player;

#[derive(Properties, PartialEq)]
pub struct InventoryScreenProps {
    pub player: Player,
    pub equipped_main_hand: Option<EquipmentItem>,
    pub equipped_off_hand: Option<EquipmentItem>,
    pub inventory_items: Vec<EquipmentItem>,
    pub on_equip_main: Callback<String>,
    pub on_equip_off: Callback<String>,
    pub on_unequip_main: Callback<()>,
    pub on_unequip_off: Callback<()>,
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
                    html! {
                        <>
                            <div class="equipment-name">{&item.name}</div>
                            <div class="equipment-meta">{format!("{} • Weight {}", damage_range_label(item), item.weight)}</div>
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

    let mut uneaten_fruits: Vec<_> = FRUIT_REGISTRY
        .values()
        .filter(|fruit| !props.player.has_eaten_fruit(&fruit.id))
        .cloned()
        .collect();
    uneaten_fruits.sort_by(|a, b| a.name.cmp(&b.name));

    html! {
        <div class="screen inventory-screen">
            <div class="inventory-header">
                <h1>{"Inventory"}</h1>
                <div class="player-stat">{format!("{} — Lv. {}", props.player.name, props.player.level)}</div>
            </div>

            <div class="inventory-body">
                <section class="inventory-section">
                    <h2>{"Equipped"}</h2>
                    <div class="equipped-grid">
                        {render_equipped_slot("Main Hand", &props.equipped_main_hand, on_unequip_main)}
                        {render_equipped_slot("Off Hand", &props.equipped_off_hand, on_unequip_off)}
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

                                        let can_main = item.can_equip_in(EquipmentSlot::MainHand);
                                        let can_off = item.can_equip_in(EquipmentSlot::OffHand);
                                        let main_disabled = props
                                            .equipped_main_hand
                                            .as_ref()
                                            .map(|equipped| equipped.id == item.id)
                                            .unwrap_or(false)
                                            || !can_main;
                                        let off_disabled = props
                                            .equipped_off_hand
                                            .as_ref()
                                            .map(|equipped| equipped.id == item.id)
                                            .unwrap_or(false)
                                            || !can_off;

                                        html! {
                                            <div class="equipment-item-card">
                                                <div class="equipment-name">{&item.name}</div>
                                                <div class="equipment-meta">{format!("{} • Weight {}", damage_range_label(item), item.weight)}</div>
                                                <p class="equipment-description">{&item.description}</p>
                                                <div class="equipment-actions">
                                                    <button class="btn btn-secondary" onclick={on_equip_main} disabled={main_disabled}>{"Equip Main"}</button>
                                                    <button class="btn btn-secondary" onclick={on_equip_off} disabled={off_disabled}>{"Equip Off"}</button>
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
                                if uneaten_fruits.is_empty() {
                                    html! { <div class="consumable-empty">{"None"}</div> }
                                } else {
                                    html! {
                                        <div class="consumable-fruit-list">
                                            {for uneaten_fruits.iter().map(|fruit| {
                                                html! {
                                                    <div class="consumable-fruit-row">
                                                        <div class="consumable-fruit-name">{&fruit.name}</div>
                                                        <div class="consumable-fruit-desc">{&fruit.description}</div>
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
