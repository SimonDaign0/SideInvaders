use defmt::{ info, println };
use embedded_graphics::{ Drawable, Pixel, image::Image, pixelcolor::BinaryColor, prelude::Point };
use esp_hal::{ Async, i2c::master::I2c, time::{ Instant, Duration } };
use ssd1306::{
    Ssd1306,
    mode::BufferedGraphicsMode,
    prelude::I2CInterface,
    size::DisplaySize128x64,
};
use super::sprites::{ HEART, SHIP, ENEMY };
const DISPLAY_WIDTH: i32 = 128;
const DISPLAY_HEIGHT: i32 = 64;
pub enum Event {
    BtnPressed(u8),
    BtnReleased(u8),
}

#[derive(PartialEq, Eq)]
pub enum State {
    Idle,
    Listening,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
pub struct StateMachine {
    pub state: State,
    pub player: Player,
    pub projectiles: [Option<Projectile>; 10],
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            state: State::Listening,
            player: Player::new(),
            projectiles: [None; 10],
        }
    }

    pub fn event_handler(&mut self, event: Event) {
        if self.state == State::Idle {
            return;
        }
        match event {
            Event::BtnPressed(1) => {
                self.player.move_pos(Direction::Left);
                self.player.move_pos(Direction::Left);
            }

            Event::BtnPressed(2) => {
                self.player.move_pos(Direction::Right);
                self.player.move_pos(Direction::Right);
            }
            Event::BtnPressed(3) => {
                self.player.move_pos(Direction::Down);
                self.player.move_pos(Direction::Down);
            }
            Event::BtnPressed(4) => {
                self.player.move_pos(Direction::Up);
                self.player.move_pos(Direction::Up);
            }
            Event::BtnPressed(5) => {
                if self.player.last_shot.elapsed() > Duration::from_millis(200) {
                    let shot_pos = if self.player.next_shot == Direction::Left {
                        self.player.next_shot = Direction::Right;
                        Coord::new(
                            self.player.pos.x,
                            self.player.pos.y + self.player.height / 2 + 5
                        )
                    } else {
                        self.player.next_shot = Direction::Left;
                        Coord::new(
                            self.player.pos.x,
                            self.player.pos.y + self.player.height / 2 - 5
                        )
                    };
                    self.spawn_projectile(Projectile::new(shot_pos, Direction::Left));
                    self.player.last_shot = Instant::now();
                }
            }

            _ => (),
        }
    }

    pub fn update(
        &mut self,
        display: &mut Ssd1306<
            I2CInterface<I2c<'_, Async>>,
            DisplaySize128x64,
            BufferedGraphicsMode<DisplaySize128x64>
        >
    ) {
        //clear
        display.clear_buffer();
        //ENEMIES
        startup_frame(display);
        //ship
        draw_ship(display, Coord::new(self.player.pos.x, self.player.pos.y));
        //health
        for i in 0..self.player.hp as i32 {
            let x = i * 8 + i;
            draw_heart(display, Coord::new(x, 0));
        }
        //projectiles
        for slot in self.projectiles.iter_mut() {
            if let Some(projectile) = slot.as_mut() {
                projectile.move_pos(projectile.direction);
                if is_out_of_bounds(&projectile.pos, 1, 1) {
                    *slot = None;
                } else {
                    draw_projectile(display, projectile.pos);
                }
            }
        }
        // flush
        _ = display.flush();
    }

    fn spawn_projectile(&mut self, projectile: Projectile) {
        for slot in &mut self.projectiles {
            if slot.is_none() {
                *slot = Some(projectile);
                break;
            }
        }
    }
}

fn draw_heart(
    display: &mut Ssd1306<
        I2CInterface<I2c<'_, Async>>,
        DisplaySize128x64,
        BufferedGraphicsMode<DisplaySize128x64>
    >,
    coord: Coord
) {
    let heart = Image::new(&HEART, Point::new(coord.x, coord.y));
    heart.draw(display).unwrap();
}

fn draw_ship(
    display: &mut Ssd1306<
        I2CInterface<I2c<'_, Async>>,
        DisplaySize128x64,
        BufferedGraphicsMode<DisplaySize128x64>
    >,
    coord: Coord
) {
    let ship = Image::new(&SHIP, Point::new(coord.x, coord.y));
    ship.draw(display).unwrap();
}

fn draw_projectile(
    display: &mut Ssd1306<
        I2CInterface<I2c<'_, Async>>,
        DisplaySize128x64,
        BufferedGraphicsMode<DisplaySize128x64>
    >,
    pos: Coord
) {
    let projectile = Pixel(Point::new(pos.x, pos.y), BinaryColor::On);
    projectile.draw(display).unwrap();
}

pub struct Player {
    hp: u8,
    pos: Coord,
    width: i32,
    height: i32,
    next_shot: Direction,
    last_shot: Instant,
}
impl Player {
    pub fn new() -> Self {
        Self {
            pos: Coord::new(DISPLAY_WIDTH / 2, DISPLAY_HEIGHT / 2),
            hp: 3,
            width: 16,
            height: 16,
            next_shot: Direction::Left,
            last_shot: Instant::now(),
        }
    }

    pub fn move_pos(&mut self, direction: Direction) {
        let mut new_pos = Coord::new(self.pos.x, self.pos.y);
        match direction {
            Direction::Up => {
                new_pos.y -= 1;
            }
            Direction::Down => {
                new_pos.y += 1;
            }
            Direction::Left => {
                new_pos.x -= 1;
            }
            Direction::Right => {
                new_pos.x += 1;
            }
        }
        if !is_out_of_bounds(&new_pos, self.width, self.height) {
            self.pos = new_pos;
        }
    }
}

#[derive(Clone, Copy)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}
impl Coord {
    fn new(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Projectile {
    pos: Coord,
    direction: Direction,
}
impl Projectile {
    fn new(pos: Coord, direction: Direction) -> Self {
        Self {
            pos,
            direction,
        }
    }
    pub fn move_pos(&mut self, direction: Direction) {
        let mut new_pos = Coord::new(self.pos.x, self.pos.y);
        match direction {
            Direction::Left => {
                new_pos.x -= 1;
            }
            Direction::Right => {
                new_pos.x += 1;
            }
            _ => (),
        }
        self.pos = new_pos;
    }
}

fn is_out_of_bounds(pos: &Coord, width: i32, height: i32) -> bool {
    pos.y < 0 ||
        pos.y + height - 1 >= DISPLAY_HEIGHT ||
        pos.x < 0 ||
        pos.x + width - 1 >= DISPLAY_WIDTH
}

fn startup_frame(
    display: &mut Ssd1306<
        I2CInterface<I2c<'_, Async>>,
        DisplaySize128x64,
        BufferedGraphicsMode<DisplaySize128x64>
    >
) {
    let enemy = Image::new(&ENEMY, Point::new(0, 39));
    enemy.draw(display).unwrap();
    let enemy = Image::new(&ENEMY, Point::new(0, 20));
    enemy.draw(display).unwrap();
}
