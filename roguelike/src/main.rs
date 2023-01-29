use map_indexing_system::MapIndexingSystem;
use rltk::{GameState, Point, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;
use specs_derive::Component;
use std::cmp::{max, min};

mod components;
mod map;
mod map_indexing_system;
mod monster_ai_system;
mod visibility_system;

use components::{BlocksTile, Monster, Name};
use map::{draw_map, rand_map_rooms_and_corridors, Map, TileType};
use monster_ai_system::MonsterAI;
use visibility_system::VisibilitySystem;

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50().with_title("Rogue").build()?;
    let mut gs = State {
        ecs: World::new(),
        runstate: RunState::Running,
    };
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
            .with(BlocksTile {})
            .build();
    }

    gs.ecs.insert(map);

    gs.ecs
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
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .build();

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
    let map = ecs.fetch::<Map>();
    for (_player, pos, viewshed) in (&mut players, &mut positions, &mut viewsheds).join() {
        let dest = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

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
    Paused,
    Running,
}

struct State {
    ecs: World,
    runstate: RunState,
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
        let mut mob = MonsterAI {};
        mob.run_now(&self.ecs);
        let mut mapindex = MapIndexingSystem {};
        mapindex.run_now(&self.ecs);
        self.ecs.maintain();
    }

    fn player_input(&mut self, ctx: &mut Rltk) -> RunState {
        use RunState::*;
        use VirtualKeyCode::*;
        match ctx.key {
            None => return Paused,
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
                _ => return Paused,
            },
        }
        Running
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

        if self.runstate == RunState::Running {
            self.run_systems();
            self.runstate = RunState::Paused;
        } else {
            self.runstate = self.player_input(ctx);
        }

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
