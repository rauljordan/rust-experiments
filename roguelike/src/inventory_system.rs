use super::{CombatStats, MeleeIntent, Name, Position, SufferDamage};
use crate::{
    components::{InBackpack, Potion, WantsToDrinkPotion, WantsToDropItem, WantsToPickupItem},
    gamelog::GameLog,
};
use rltk::console;
use specs::prelude::*;

pub struct PotionUseSystem {}

impl<'a> System<'a> for PotionUseSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToDrinkPotion>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Potion>,
        WriteStorage<'a, CombatStats>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_ent, mut gamelog, entities, mut wants_drink, names, potions, mut combat_stats) =
            data;

        for (entity, drink, stats) in (&entities, &wants_drink, &mut combat_stats).join() {
            match potions.get(drink.potion) {
                None => {}
                Some(potion) => {
                    stats.hp = i32::min(stats.max_hp, stats.hp + potion.heal_amount);
                    if entity == *player_ent {
                        gamelog.entries.push(format!(
                            "You drink {}, healing {}",
                            names.get(drink.potion).unwrap().name,
                            potion.heal_amount
                        ));
                    }
                    entities
                        .delete(drink.potion)
                        .expect("could not delete entity");
                }
            }
        }
        wants_drink.clear();
    }
}

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToDropItem>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_ent, mut gamelog, entities, mut wants_drop, names, mut positions, mut backpack) =
            data;

        for (entity, to_drop) in (&entities, &wants_drop).join() {
            let mut dropper_pos: Position = Position { x: 0, y: 0 };
            {
                let dropped_pos = positions.get(entity).unwrap();
                dropper_pos.x = dropped_pos.x;
                dropper_pos.x = dropped_pos.x;
            }
            positions
                .insert(
                    to_drop.item,
                    Position {
                        x: dropper_pos.x,
                        y: dropper_pos.y,
                    },
                )
                .expect("Unable to insert");
            backpack.remove(to_drop.item);
            if entity == *player_ent {
                gamelog.entries.push(format!(
                    "You drop {}",
                    names.get(to_drop.item).unwrap().name
                ));
            }
        }
    }
}

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToPickupItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, mut wants_pickup, mut positions, names, mut backpack) =
            data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack
                .insert(
                    pickup.item,
                    InBackpack {
                        owner: pickup.collected_by,
                    },
                )
                .expect("unable to insert backpack");

            if pickup.collected_by == *player_entity {
                gamelog.entries.push(format!(
                    "You pickup the {}",
                    names.get(pickup.item).unwrap().name
                ));
            }
        }
        wants_pickup.clear();
    }
}
