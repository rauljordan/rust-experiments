use specs::prelude::*;
use specs_derive::*;

#[derive(Component, Debug)]
pub struct Name {
    pub name: String,
}

#[derive(Component, Debug)]
pub struct Monster;

#[derive(Component, Debug)]
pub struct BlocksTile {}

#[derive(Component, Debug)]
pub struct MeleeIntent {
    pub target: Entity,
}

#[derive(Component, Debug)]
pub struct SufferDamage {
    pub amount: Vec<i32>,
}

impl SufferDamage {
    pub fn new_damage(store: &mut WriteStorage<SufferDamage>, victim: Entity, amount: i32) {
        match store.get_mut(victim) {
            Some(suffering) => {
                suffering.amount.push(amount);
            }
            None => {
                let dmg = SufferDamage {
                    amount: vec![amount],
                };
                store.insert(victim, dmg).expect("unable to insert");
            }
        }
    }
}

#[derive(Component, Debug)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
}
