use damage_system::DamageSystem;
use inventory_system::{ItemCollectionSystem, ItemDropSystem, PotionUseSystem};
use map_indexing_system::MapIndexingSystem;
use melee_combat_system::MeleeCombatSystem;
use rltk::{console, GameState, Point, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;
use specs_derive::Component;
use std::cmp::{max, min};

mod components;
mod damage_system;
mod gamelog;
mod inventory_system;
mod map;
mod map_indexing_system;
mod melee_combat_system;
mod monster_ai_system;
mod spawner;
mod ui;
mod visibility_system;

use components::{BlocksTile, CombatStats, Monster, Name, WantsToDrinkPotion};
use map::{draw_map, rand_map_rooms_and_corridors, Map, TileType};
use monster_ai_system::MonsterAI;
use visibility_system::VisibilitySystem;

use crate::components::{
    InBackpack, Item, MeleeIntent, Potion, SufferDamage, WantsToDropItem, WantsToPickupItem,
};

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let mut context = RltkBuilder::simple80x50().with_title("Rogue").build()?;
    context.with_post_scanlines(true); // Add burn.
    let mut gs = State { ecs: World::new() };
    macro_rules! reg {
        ($name:ident) => {
            gs.ecs.register::<$name>();
        };
    }
    reg!(Renderable);
    reg!(Position);
    reg!(LeftMover);
    reg!(Player);
    reg!(Viewshed);
    reg!(Map);
    reg!(Monster);
    reg!(Name);
    reg!(BlocksTile);
    reg!(CombatStats);
    reg!(MeleeIntent);
    reg!(SufferDamage);
    reg!(Item);
    reg!(Potion);
    reg!(InBackpack);
    reg!(WantsToPickupItem);
    reg!(WantsToDrinkPotion);
    reg!(WantsToDropItem);

    let map = rand_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();

    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, room);
    }

    gs.ecs.insert(map);
    gs.ecs.insert(RunState::PreRun);

    let player_entity = spawner::player(&mut gs.ecs, player_x, player_y);

    gs.ecs.insert(player_entity);
    gs.ecs.insert(Point::new(player_x, player_y));
    gs.ecs.insert(gamelog::GameLog {
        entries: vec!["Welcome to roguelike".to_string()],
    });

    rltk::main_loop(context, gs)
}

#[derive(Component)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32,
    pub dirty: bool,
}

#[derive(Component, Debug)]
pub struct Player {}

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let mut ppos = ecs.write_resource::<Point>();
    let mut intent_to_melee = ecs.write_storage::<MeleeIntent>();

    let combat_stats = ecs.read_storage::<CombatStats>();
    let map = ecs.fetch::<Map>();
    let entities = ecs.entities();

    for (entity, _player, pos, viewshed) in
        (&entities, &mut players, &mut positions, &mut viewsheds).join()
    {
        if pos.x + delta_x < 1
            || pos.x + delta_x > map.width - 1
            || pos.y + delta_y < 1
            || pos.y + delta_y > map.height - 1
        {
            return;
        }
        let dest = map.xy_idx(pos.x + delta_x, pos.y + delta_y);
        for potential_target in map.tile_content[dest].iter() {
            let target = combat_stats.get(*potential_target);
            if let Some(_target) = target {
                intent_to_melee
                    .insert(
                        entity,
                        MeleeIntent {
                            target: *potential_target,
                        },
                    )
                    .expect("Add target failed");
                return;
            }
        }
        if !map.blocked[dest] {
            pos.x = min(79, max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));

            ppos.x = pos.x;
            ppos.y = pos.y;
            viewshed.dirty = true;
        }
    }
}
struct LeftWalker {}

impl<'a> System<'a> for LeftWalker {
    type SystemData = (ReadStorage<'a, LeftMover>, WriteStorage<'a, Position>);

    fn run(&mut self, (lefty, mut pos): Self::SystemData) {
        for (_lefty, pos) in (&lefty, &mut pos).join() {
            pos.x -= 1;
            if pos.x < 0 {
                pos.x = 79;
            }
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem,
}

pub struct State {
    pub ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
        let mut mob = MonsterAI {};
        mob.run_now(&self.ecs);
        let mut mapindex = MapIndexingSystem {};
        mapindex.run_now(&self.ecs);
        let mut melee = MeleeCombatSystem {};
        melee.run_now(&self.ecs);
        let mut dmg = DamageSystem {};
        dmg.run_now(&self.ecs);
        let mut pickup = ItemCollectionSystem {};
        pickup.run_now(&self.ecs);
        let mut potions = PotionUseSystem {};
        potions.run_now(&self.ecs);
        let mut drop_items = ItemDropSystem {};
        drop_items.run_now(&self.ecs);
        self.ecs.maintain();
    }

    fn player_input(&mut self, ctx: &mut Rltk) -> RunState {
        use RunState::*;
        use VirtualKeyCode::*;
        match ctx.key {
            None => return AwaitingInput,
            Some(key) => match key {
                H => try_move_player(-1, 0, &mut self.ecs),
                L => try_move_player(1, 0, &mut self.ecs),
                K => try_move_player(0, -1, &mut self.ecs),
                J => try_move_player(0, 1, &mut self.ecs),

                // Diagonal.
                B => try_move_player(-1, 1, &mut self.ecs),
                O => try_move_player(1, -1, &mut self.ecs),
                M => try_move_player(1, 1, &mut self.ecs),
                Y => try_move_player(-1, -1, &mut self.ecs),

                G => get_item(&mut self.ecs),
                I => return ShowInventory,
                D => return ShowDropItem,
                _ => return AwaitingInput,
            },
        }
        PlayerTurn
    }
}

fn get_item(ecs: &mut World) {
    let player_pos = ecs.fetch::<Point>();
    let player_ent = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.fetch_mut::<gamelog::GameLog>();

    let mut target_item: Option<Entity> = None;

    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => gamelog.entries.push("Nothing here to pickup".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup
                .insert(
                    *player_ent,
                    WantsToPickupItem {
                        collected_by: *player_ent,
                        item,
                    },
                )
                .expect("unable to insert want to pickup");
        }
    }
}

pub fn delete_the_dead(ecs: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();
    {
        let combat_stats = ecs.read_storage::<CombatStats>();
        let players = ecs.read_storage::<Player>();
        let names = ecs.read_storage::<Name>();
        let entities = ecs.entities();
        let mut log = ecs.write_resource::<gamelog::GameLog>();
        for (entity, stats) in (&entities, &combat_stats).join() {
            if stats.hp < 1 {
                match players.get(entity) {
                    None => {
                        if let Some(victim_name) = names.get(entity) {
                            log.entries.push(format!("{} is dead", &victim_name.name));
                        }
                        dead.push(entity);
                    }
                    Some(_) => console::log("You are dead"),
                }
            }
        }
    }
    for victim in dead {
        ecs.delete_entity(victim).expect("unable to delete");
    }
}

#[derive(Component)]
pub struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Renderable {
    glyph: rltk::FontCharType,
    fg: RGB,
    bg: RGB,
    pub render_order: i32,
}

#[derive(Component)]
struct LeftMover {}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        match newrunstate {
            RunState::PreRun => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = self.player_input(ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::ShowInventory => {
                let result = ui::show_inventory(self, ctx);
                match result.0 {
                    ui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    ui::ItemMenuResult::NoResponse => {}
                    ui::ItemMenuResult::Selected => {
                        let item_ent = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDrinkPotion>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToDrinkPotion { potion: item_ent },
                            )
                            .expect("Could not insert pot intent");
                        newrunstate = RunState::PlayerTurn;
                    }
                }
            }
            RunState::ShowDropItem => {
                let result = ui::drop_item_menu(self, ctx);
                match result.0 {
                    ui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    ui::ItemMenuResult::NoResponse => {}
                    ui::ItemMenuResult::Selected => {
                        let mut item_ent = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToDropItem { item: item_ent },
                            )
                            .expect("Unable");
                        newrunstate = RunState::PlayerTurn;
                    }
                }
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }

        delete_the_dead(&mut self.ecs);

        draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
        data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));
        for (pos, render) in data.iter() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }

        ui::draw_ui(&self.ecs, ctx);
    }
}
