use damage_system::DamageSystem;
use map_indexing_system::MapIndexingSystem;
use melee_combat_system::MeleeCombatSystem;
use rltk::{console, GameState, Point, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;
use specs_derive::Component;
use std::cmp::{max, min};

mod components;
mod damage_system;
mod map;
mod map_indexing_system;
mod melee_combat_system;
mod monster_ai_system;
mod visibility_system;

use components::{BlocksTile, CombatStats, Monster, Name};
use map::{draw_map, rand_map_rooms_and_corridors, Map, TileType};
use monster_ai_system::MonsterAI;
use visibility_system::VisibilitySystem;

use crate::components::{MeleeIntent, SufferDamage};

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50().with_title("Rogue").build()?;
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

    let map = rand_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();

    let mut rng = rltk::RandomNumberGenerator::new();

    for (i, room) in map.rooms.iter().skip(1).enumerate() {
        let (x, y) = room.center();
        let roll = rng.roll_dice(1, 2);
        let name: String;
        let glyph: rltk::FontCharType;
        match roll {
            1 => {
                name = "Goblin".to_string();
                glyph = rltk::to_cp437('o');
            }
            _ => {
                name = "Org".to_string();
                glyph = rltk::to_cp437('m');
            }
        }
        gs.ecs
            .create_entity()
            .with(Position { x, y })
            .with(Renderable {
                glyph,
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .with(Viewshed {
                visible_tiles: Vec::new(),
                range: 8,
                dirty: true,
            })
            .with(Monster {})
            .with(Name {
                name: format!("{} #{}", &name, i),
            })
            .with(CombatStats {
                max_hp: 15,
                hp: 15,
                defense: 1,
                power: 3,
            })
            .with(BlocksTile {})
            .build();
    }

    gs.ecs.insert(map);
    gs.ecs.insert(RunState::PreRun);

    let player_entity = gs
        .ecs
        .create_entity()
        .with(Position {
            x: player_x,
            y: player_y,
        })
        .with(Renderable {
            glyph: rltk::to_cp437('X'),
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .with(Name {
            name: "player".to_string(),
        })
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .build();

    gs.ecs.insert(player_entity);
    gs.ecs.insert(Point::new(player_x, player_y));

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
}

struct State {
    ecs: World,
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
                _ => return AwaitingInput,
            },
        }
        PlayerTurn
    }
}

pub fn delete_the_dead(ecs: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();
    {
        let combat_stats = ecs.read_storage::<CombatStats>();
        let players = ecs.read_storage::<Player>();
        let entities = ecs.entities();
        for (entity, stats) in (&entities, &combat_stats).join() {
            if stats.hp < 1 {
                match players.get(entity) {
                    None => dead.push(entity),
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
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = self.player_input(ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                newrunstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
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
        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }
    }
}
