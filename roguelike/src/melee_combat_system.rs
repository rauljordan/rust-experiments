use super::{CombatStats, MeleeIntent, Name, SufferDamage};
use rltk::console;
use specs::prelude::*;

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, MeleeIntent>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut wants_melee, names, combat_stats, mut inflict) = data;

        for (_entity, wants_melee, name, stats) in
            (&entities, &wants_melee, &names, &combat_stats).join()
        {
            if stats.hp <= 0 {
                continue;
            }
            let target_stats = combat_stats.get(wants_melee.target).unwrap();
            if target_stats.hp <= 0 {
                continue;
            }
            let target_name = names.get(wants_melee.target).unwrap();
            let damage = i32::max(0, stats.power - target_stats.defense);

            if damage == 0 {
                console::log(&format!(
                    "{} is unable to hurt {}",
                    &name.name, &target_name.name
                ));
            } else {
                console::log(&format!(
                    "{} hits {}, for {} hp.",
                    &name.name, &target_name.name, damage
                ));
                SufferDamage::new_damage(&mut inflict, wants_melee.target, damage);
            }
        }
    }
}
