use raylib::prelude::*;
use raylib_sys::TraceLogLevel;
use std::{cell::OnceCell, ffi::CString};

const SCREEN_WIDTH: i32 = 1200;
const SCREEN_HEIGHT: i32 = 650;
const PAINT_RADIUS: f32 = 5.0; // Radius of the paint splat

// global counter

#[derive(Debug)]
pub struct Player {
    pub position: Vector2,
    pub velocity: Vector2,
    pub rotation: f32,
    pub speed: f32,
    pub color: Color,
    pub controls: Controls,
    pub game: Box<MiniGames>,
    pub is_on_ground: bool,
    pub width: f32,
    pub height: f32,
    pub jump_force: f32,
    pub texture: Texture2D,
    pub is_jumping: bool,
    pub jump_time: f32,
    pub max_jump_time: f32,
    pub min_jump_velocity: f32,
    pub points: u32,
}

#[derive(Debug, Copy, Clone)]
pub enum Controls {
    WASD,
    ArrowKeys,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MiniGames {
    ColorTheMap,
    Dodge,
    FloorIsLava,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]

pub enum GameMode {
    MainMenu,
    Game,
    WinScreen,
}

pub struct Input {
    pub up: consts::KeyboardKey,
    pub down: consts::KeyboardKey,
    pub left: consts::KeyboardKey,
    pub right: consts::KeyboardKey,
    pub primary: consts::KeyboardKey,
    pub secondary: consts::KeyboardKey,
}

impl Player {
    pub fn new(
        position: Vector2,
        rotation: f32,
        speed: f32,
        color: Color,
        controls: Controls,
        game: Box<MiniGames>,
        width: f32,
        height: f32,
        jump_force: f32,
        texture: Texture2D,
    ) -> Self {
        Player {
            position,
            rotation,
            speed,
            color,
            velocity: Vector2::zero(),
            controls,
            game,
            is_on_ground: false,
            width,
            height,
            jump_force,
            texture,
            is_jumping: false,
            jump_time: 0.0,
            max_jump_time: 0.4, // Maximum time the jump can be held (in seconds)
            min_jump_velocity: 200.0, // Minimum jump velocity when tapping
            points: 0,
        }
    }

    pub fn update(&mut self, rl: &RaylibHandle, dt: f32) {
        let keys: Input;

        match self.controls {
            Controls::WASD => {
                keys = Input {
                    up: consts::KeyboardKey::KEY_W,
                    down: consts::KeyboardKey::KEY_S,
                    left: consts::KeyboardKey::KEY_A,
                    right: consts::KeyboardKey::KEY_D,
                    primary: consts::KeyboardKey::KEY_F,
                    secondary: consts::KeyboardKey::KEY_G,
                };
            }

            Controls::ArrowKeys => {
                keys = Input {
                    up: consts::KeyboardKey::KEY_UP,
                    down: consts::KeyboardKey::KEY_DOWN,
                    left: consts::KeyboardKey::KEY_LEFT,
                    right: consts::KeyboardKey::KEY_RIGHT,
                    primary: consts::KeyboardKey::KEY_J,
                    secondary: consts::KeyboardKey::KEY_K,
                };
            }
        }
        // Apply gravity.  This happens *before* jump input.
        if !self.is_on_ground {
            self.velocity.y += 980.8 * dt;
        }

        // New jump logic
        if *self.game == MiniGames::ColorTheMap {
            if rl.is_key_down(keys.up) && self.is_on_ground && !self.is_jumping {
                self.velocity.y = -self.jump_force;
                self.is_jumping = true;
                self.jump_time = 0.0;
                self.is_on_ground = false;
            } else if rl.is_key_down(keys.up) && self.is_jumping {
                self.jump_time += dt;
                if self.jump_time < self.max_jump_time {
                    // Continue applying upward force while holding jump
                    self.velocity.y =
                        -self.jump_force * (1.0 - (self.jump_time / self.max_jump_time));
                }
            } else if self.is_jumping {
                // Player released jump button or exceeded max jump time
                self.is_jumping = false;
                if self.velocity.y < -self.min_jump_velocity {
                    self.velocity.y = -self.min_jump_velocity;
                }
            }
        }

        let mut horizontal_input = 0.0;
        if rl.is_key_down(keys.right) {
            horizontal_input += 1.0;
        }
        if rl.is_key_down(keys.left) {
            horizontal_input -= 1.0;
        }

        match *self.game {
            MiniGames::ColorTheMap => {
                self.velocity.x = horizontal_input * self.speed;
            }

            _ => {}
        }

        self.position += self.velocity * dt;
    }
    pub fn handle_collision(&mut self, ops: &Vec<EnvItem>) -> Vec<(Rectangle, Vec<Vector2>)> {
        let player_rect = self.get_collision_rect();
        let mut collisions = Vec::new();

        for op in ops {
            if let Some(collision) = player_rect.get_collision_rec(&op.rect) {
                // Resolve collision
                let dx = collision.width;
                let dy = collision.height;

                if dx < dy {
                    // X-axis collision
                    if player_rect.x < op.rect.x {
                        self.position.x -= dx;
                    } else {
                        self.position.x += dx;
                    }
                    self.velocity.x = 0.0;
                } else {
                    // Y-axis collision
                    if player_rect.y < op.rect.y {
                        self.position.y -= dy;
                        self.velocity.y = 0.0;
                        self.is_on_ground = true;
                    } else {
                        self.position.y += dy;
                        self.velocity.y = 0.0;
                    }
                }

                // Generate collision points
                let mut points = Vec::new();
                let step = PAINT_RADIUS * 1.0;

                let start_x = collision.x;
                let end_x = collision.x + collision.width;
                let start_y = collision.y;
                let end_y = collision.y + collision.height;

                let mut x = start_x;
                while x < end_x {
                    let mut y = start_y;
                    while y < end_y {
                        let adjusted_x = x + PAINT_RADIUS;
                        let adjusted_y = y + PAINT_RADIUS;
                        points.push(Vector2::new(adjusted_x, adjusted_y));
                        y += step;
                    }
                    x += step;
                }

                // Ensure at least one point for small collisions
                if points.is_empty() {
                    let center_x = collision.x + collision.width / 2.0 + PAINT_RADIUS;
                    let center_y = collision.y + collision.height / 2.0 + PAINT_RADIUS;
                    points.push(Vector2::new(center_x, center_y));
                }

                collisions.push((op.rect.clone(), points));
            }
        }

        collisions
    }

    pub fn get_collision_rect(&self) -> Rectangle {
        Rectangle {
            x: self.position.x - self.width / 2.0,
            y: self.position.y - self.height / 2.0,
            width: self.width,
            height: self.height,
        }
    }

    pub fn draw(&self, d: &mut RaylibMode2D<'_, RaylibDrawHandle>) {
        // d.draw_rectangle_pro(
        //     Rectangle {
        //         x: self.position.x,
        //         y: self.position.y,
        //         width: self.width,
        //         height: self.height,
        //     },
        //     Vector2::new(self.width / 2.0, self.height / 2.0),
        //     self.rotation,
        //     self.color,
        // );
        d.draw_texture_ex(
            &self.texture,
            Vector2::new(
                self.position.x - self.width / 2.,
                self.position.y - self.height / 2.,
            ),
            self.rotation,
            0.32,
            Color::WHITE,
        );
    }
    // Modified paint function
    pub fn paint(&self, image: &mut Image, collision_point: Vector2) {
        // Use the collision point for drawing.  Offset by radius to center the circle.
        let image_x = (collision_point.x - PAINT_RADIUS).round() as i32;
        let image_y = (collision_point.y - PAINT_RADIUS).round() as i32;
        image.draw_circle(image_x, image_y, PAINT_RADIUS as i32, self.color);
    }
}

pub struct EnvItem {
    pub rect: Rectangle,
    pub color: Color,
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Color The Map")
        .resizable()
        .build();
    let mut trantition_right_image = Image::load_image("./static/transition_right.png").unwrap();
    trantition_right_image.resize(SCREEN_WIDTH / 2, SCREEN_HEIGHT);

    let mut level_timer = 30.0;
    let trantition_right_texture = rl
        .load_texture_from_image(&thread, &trantition_right_image)
        .unwrap();
    let mut trantition_left_image = Image::load_image("./static/transition_left.png").unwrap(); // Load image data into CPU memory (RAM)
    trantition_left_image.resize(SCREEN_WIDTH / 2, SCREEN_HEIGHT);
    let trantition_left_texture = rl
        .load_texture_from_image(&thread, &trantition_left_image)
        .unwrap();
    let mut player1_texture = rl.load_texture(&thread, "./static/player1.png").unwrap();
    let mut player2_texture = rl.load_texture(&thread, "./static/player2.png").unwrap();

    let mut level_image = Image::load_image("./static/level.png").unwrap();
    level_image.resize(SCREEN_WIDTH, SCREEN_HEIGHT);
    let mut level_texture = rl.load_texture_from_image(&thread, &level_image).unwrap();
    let mut trantition_progress = 0.0;
    let mut transitioning = false;
    let mut reversing = false;
    let mut in_game = false;
    let mut delay_timer = 0.0;
    let mut head_msg: Option<String> = None;
    let mut level_done = false;
    let mut level_end_timer = 5.0;

    let mut game_type = Box::new(MiniGames::ColorTheMap);
    let mut game_mode = GameMode::MainMenu;

    let mut camera = Camera2D {
        offset: Vector2::new(
            (rl.get_screen_width() as f32 / 2.0) - SCREEN_WIDTH as f32 / 2.,
            (rl.get_screen_height() as f32 / 2.0) - SCREEN_HEIGHT as f32 / 2.,
        ),
        zoom: 1.0,
        ..Default::default()
    };

    let mut ops: Vec<EnvItem> = vec![
        EnvItem {
            rect: Rectangle {
                x: 0.0,
                y: 0.0,
                width: SCREEN_WIDTH as f32,
                height: 30.0,
            },
            color: Color::RED.alpha(0.5),
        },
        EnvItem {
            rect: Rectangle {
                x: SCREEN_WIDTH as f32 - 15.0,
                y: 50.0,
                width: 15.0,
                height: 120.,
            },
            color: Color::RED.alpha(0.5),
        },
        EnvItem {
            rect: Rectangle {
                x: SCREEN_WIDTH as f32 - 15.0,
                y: 240.0,
                width: 15.0,
                height: 120.,
            },
            color: Color::RED.alpha(0.5),
        },
        EnvItem {
            rect: Rectangle {
                x: SCREEN_WIDTH as f32 - 15.0,
                y: 425.0,
                width: 15.0,
                height: 90.,
            },
            color: Color::RED.alpha(0.5),
        },
        EnvItem {
            rect: Rectangle {
                x: 0.0,
                y: 45.0,
                width: 15.0,
                height: 45.,
            },
            color: Color::RED.alpha(0.5),
        },
        EnvItem {
            rect: Rectangle {
                x: 0.0,
                y: 160.0,
                width: 15.0,
                height: 30.,
            },
            color: Color::RED.alpha(0.5),
        },
        EnvItem {
            rect: Rectangle {
                x: 0.0,
                y: 260.0,
                width: 15.0,
                height: 153.,
            },
            color: Color::RED.alpha(0.5),
        },
        EnvItem {
            rect: Rectangle {
                x: 0.0,
                y: 480.0,
                width: 15.0,
                height: 95.,
            },
            color: Color::RED.alpha(0.5),
        },
        EnvItem {
            rect: Rectangle {
                x: 1010.,
                y: 185.,
                width: 182.0,
                height: 30.0,
            },
            color: Color::RED.alpha(0.5),
        },
        EnvItem {
            rect: Rectangle {
                x: 9.,
                y: 119.,
                width: 117.0,
                height: 30.0,
            },
            color: Color::RED.alpha(0.5),
        },
        EnvItem {
            rect: Rectangle {
                x: 9.,
                y: 209.,
                width: 217.0,
                height: 30.0,
            },
            color: Color::RED.alpha(0.5),
        },
        EnvItem {
            rect: Rectangle {
                x: 725.,
                y: 210.,
                width: 45.0,
                height: 60.0,
            },
            color: Color::RED.alpha(0.5),
        },
        EnvItem {
            rect: Rectangle {
                x: 590.,
                y: 210.,
                width: 40.0,
                height: 60.0,
            },
            color: Color::RED.alpha(0.5),
        },
        EnvItem {
            rect: Rectangle {
                x: 450.,
                y: 260.,
                width: 460.0,
                height: 30.0,
            },
            color: Color::RED.alpha(0.5),
        },
        EnvItem {
            rect: Rectangle {
                x: 130.,
                y: 320.,
                width: 220.0,
                height: 30.0,
            },
            color: Color::RED.alpha(0.5),
        },
        EnvItem {
            rect: Rectangle {
                x: 975.,
                y: 330.,
                width: 40.0,
                height: 60.0,
            },
            color: Color::RED.alpha(0.5),
        },
        EnvItem {
            rect: Rectangle {
                x: 907.,
                y: 370.,
                width: 285.,
                height: 30.0,
            },
            color: Color::RED.alpha(0.5),
        },
        EnvItem {
            rect: Rectangle {
                x: 9.,
                y: 439.,
                width: 493.0,
                height: 30.0,
            },
            color: Color::RED.alpha(0.5),
        },
        EnvItem {
            rect: Rectangle {
                x: 655.,
                y: 485.,
                width: 395.0,
                height: 30.0,
            },
            color: Color::RED.alpha(0.5),
        },
        EnvItem {
            rect: Rectangle {
                x: SCREEN_WIDTH as f32 - 20.0 - 30.0,
                y: SCREEN_HEIGHT as f32 - 115.,
                width: 35.0,
                height: 60.0,
            },
            color: Color::RED.alpha(0.5),
        },
        EnvItem {
            rect: Rectangle {
                x: 345.0,
                y: SCREEN_HEIGHT as f32 - 115.,
                width: 50.0,
                height: 60.0,
            },
            color: Color::RED.alpha(0.5),
        },
        EnvItem {
            rect: Rectangle {
                x: 10.0,
                y: SCREEN_HEIGHT as f32 - 60.0,
                width: SCREEN_WIDTH as f32 - 20.0,
                height: 60.0,
            },
            color: Color::BLUE.alpha(0.5),
        },
    ];

    let mut players = [
        Player::new(
            Vector2::new(100.0, 100.0),
            0.0,
            200.0,
            Color::RED,
            Controls::WASD,
            game_type.clone(),
            50.0,
            50.0,
            700.0,
            player1_texture,
        ),
        Player::new(
            Vector2::new(200.0, 100.0),
            0.0,
            200.0,
            Color::GREEN,
            Controls::ArrowKeys,
            game_type.clone(),
            50.0,
            50.0,
            700.0,
            player2_texture,
        ),
    ];

    let mut map_image =
        Image::gen_image_color(SCREEN_WIDTH, SCREEN_HEIGHT, Color::WHITE.alpha(0.0));
    let mut map_texture = rl.load_texture_from_image(&thread, &map_image).unwrap();

    rl.set_target_fps(60);
    let mut persents: [f32; 4] = [0.0; 4];

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();

        // Update transition
        if transitioning {
            if !reversing {
                trantition_progress += dt * 2.0;
                if trantition_progress >= 1.0 {
                    trantition_progress = 1.0;
                    game_mode = GameMode::Game;
                    delay_timer = 0.0;
                    reversing = true;
                }
            } else {
                delay_timer += dt;
                if delay_timer >= 0.15 {
                    // Wait 1 second before reversing
                    trantition_progress -= dt * 2.0;
                    if trantition_progress <= 0.0 {
                        trantition_progress = 0.0;
                        transitioning = false;
                        reversing = false;
                    }
                }
            }
        }
        for player in &mut players {
            if !level_done {
                player.update(&rl, dt);
                let collisions = player.handle_collision(&ops);
                let is_colliding = !collisions.is_empty();

                let points: Vec<Vector2> = collisions
                    .into_iter()
                    .flat_map(|(_, collision_points)| collision_points)
                    .collect();
                for point in points {
                    player.paint(&mut map_image, point);
                }
                if !is_colliding {
                    player.is_on_ground = false;
                }
            }
        }

        let width = map_image.width;
        let height = map_image.height;
        let format = map_image.format();
        let data = unsafe {
            std::slice::from_raw_parts(
                map_image.data as *const u8,
                raylib::texture::get_pixel_data_size(width, height, format)
                    .try_into()
                    .unwrap(),
            )
        };
        // let mut reset_game = move || {
        // };

        map_texture.update_texture(data);
        if (game_mode == GameMode::Game && !level_done) {
            level_timer -= dt;
        }
        if (level_done) {
            level_end_timer -= dt;
        }
        if (level_end_timer <= 0.0) {
            level_done = false;
            level_end_timer = 5.0;
            level_timer = 30.0;
            head_msg = None;
            game_type = Box::new(MiniGames::Dodge);
        }

        if (level_timer <= 0.0) {
            // level += 1;
            persents = calculate_winner(
                &mut map_image,
                2,
                &players[0].color,
                &players[1].color,
                &Color::ALICEBLUE,
                &Color::WHEAT,
            );
            // get index of largest value
            let mut index = 0;
            for i in 0..persents.len() {
                if persents[i] > persents[index] {
                    index = i;
                }
            }
            println!("{:?}", index);

            match index {
                0 => players[0].points += 1,
                1 => players[1].points += 1,
                // 2 => players[2].points += 1,
                // 3 => players[3].points += 1,
                _ => {}
            }
            head_msg = Some(format!("player {} won", index + 1));

            for player in &mut players {
                if player.points >= 5 {
                    // player.points += 1;
                }
                // player.reset();
            }
            level_done = true;
            // level_timer = 5.0;
            // spown a corotene and after 5 seconds change the game type
            use std::thread;
            use std::time::Duration;

            // thread::spawn(move || {

            //     game_type = MiniGames::Dodge;
            // });
        }

        // --- Drawing ---
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::from_hex("C7DCD0").unwrap());

        // Add mouse position logging
        // if d.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
        //     let mouse_pos = d.get_mouse_position();
        //     println!("Mouse clicked at: x={}, y={}", mouse_pos.x, mouse_pos.y);
        // }

        // if (d.is_key_pressed(consts::KeyboardKey::KEY_ENTER)) {
        //     match calculate_winner(&mut map_image, &players[0].color, &players[1].color) {
        //         Some(1) => {
        //             players[0].color = Color::GOLD;
        //         }
        //         Some(2) => {
        //             players[1].color = Color::GOLD;
        //         }
        //         None => {
        //             // player1.color = Color::PINK;
        //             // player2.color = Color::PINK;
        //         }
        //         _ => {}
        //     }
        // }

        {
            camera.offset = Vector2::new(
                (d.get_screen_width() as f32 / 2.0) - SCREEN_WIDTH as f32 / 2.,
                (d.get_screen_height() as f32 / 2.0) - SCREEN_HEIGHT as f32 / 2.,
            );
            let mut d = d.begin_mode2D(camera);

            match game_mode {
                GameMode::Game => {
                    d.draw_texture(&level_texture, 0, 0, Color::WHITE);
                    if (game_type == Box::new(MiniGames::ColorTheMap)) {
                        d.draw_texture(&map_texture, 0, 0, Color::WHITE);
                    }
                    for player in players.iter() {
                        player.draw(&mut d);
                    }

                    // for op in ops.iter() {
                    //     d.draw_rectangle_rec(op.rect, op.color);
                    // }

                    // Keep drawing transition during game mode
                    let screen_center = SCREEN_WIDTH as f32 / 2.0;
                    let effective_progress = (trantition_progress * 2.0).min(1.0);

                    let left_x =
                        -trantition_left_image.width as f32 + (effective_progress * screen_center);
                    let right_x = SCREEN_WIDTH as f32 - (effective_progress * screen_center);

                    d.draw_texture(&trantition_left_texture, left_x as i32, 0, Color::WHITE);

                    d.draw_texture(&trantition_right_texture, right_x as i32, 0, Color::WHITE);
                    d.draw_text(
                        &(level_timer as i32).to_string(),
                        SCREEN_WIDTH / 2,
                        20,
                        35,
                        Color::BLACK,
                    );
                    if let Some(msg) = &head_msg {
                        d.draw_text(
                            &msg,
                            SCREEN_WIDTH / 2 - d.measure_text(msg, 35) / 2,
                            SCREEN_HEIGHT / 2 - 35,
                            35,
                            Color::BLACK,
                        );
                        // display the persents orders from highest to lowest with the coller of it
                        //
                        let mut orderd = persents.clone();
                        orderd.sort_by(|a, b| b.partial_cmp(a).unwrap());
                        for (i, order) in orderd.iter().enumerate() {
                            let og_index: Option<usize> = persents
                                .iter()
                                .position(|x| *x != 0. && x == order)
                                .or_else(|| None);
                            println!("{:?}", og_index);
                            if let Some(index) = og_index {
                                d.draw_text(
                                    &format!("{}: {:.1}%", i + 1, order * 100.0),
                                    SCREEN_WIDTH / 2
                                        - d.measure_text(
                                            &format!("{}: {:.1}%", i + 1, order * 100.0),
                                            20,
                                        ) / 2,
                                    SCREEN_HEIGHT / 2 + 50 + i as i32 * 20,
                                    20,
                                    // get index and get color of players
                                    players[index].color,
                                );
                            }
                        }
                    }
                }
                GameMode::WinScreen => {
                    let bounds = Rectangle::new(
                        ((SCREEN_WIDTH / 2) - 50) as f32,
                        ((SCREEN_HEIGHT / 2) - 25) as f32,
                        100.0,
                        50.0,
                    );
                    let play_button = d.gui_button(bounds, Some(rstr!("Play Again")));
                    if play_button {
                        game_mode = GameMode::Game;
                    }
                }
                GameMode::MainMenu => {
                    let bounds = Rectangle::new(
                        ((SCREEN_WIDTH / 2) - 50) as f32,
                        ((SCREEN_HEIGHT / 2) - 25) as f32,
                        100.0,
                        50.0,
                    );

                    let play_button = d.gui_button(bounds, Some(rstr!("Play")));

                    // Draw transition textures
                    if transitioning {
                        let screen_center = SCREEN_WIDTH as f32 / 2.0;
                        let effective_progress = (trantition_progress * 2.0).min(1.0);

                        let left_x = -trantition_left_image.width as f32
                            + (effective_progress * screen_center);
                        let right_x = SCREEN_WIDTH as f32 - (effective_progress * screen_center);

                        d.draw_texture(&trantition_left_texture, left_x as i32, 0, Color::WHITE);

                        d.draw_texture(&trantition_right_texture, right_x as i32, 0, Color::WHITE);
                    }

                    if play_button && !transitioning {
                        transitioning = true;
                        reversing = false;
                    }
                }
            }
        }
    }
}

fn calculate_winner(
    image: &mut Image,
    players_count: usize,
    player1_color: &Color,
    player2_color: &Color,
    player3_color: &Color,
    player4_color: &Color,
) -> [f32; 4] {
    let mut player1_count = 0;
    let mut player2_count = 0;
    let mut player3_count = 0;
    let mut player4_count = 0;

    for y in 0..image.height() {
        for x in 0..image.width() {
            let pixel_color = image.get_color(x, y);
            if pixel_color.r == player1_color.r
                && pixel_color.g == player1_color.g
                && pixel_color.b == player1_color.b
            {
                player1_count += 1;
            } else if pixel_color.r == player2_color.r
                && pixel_color.g == player2_color.g
                && pixel_color.b == player2_color.b
            {
                player2_count += 1;
            } else if players_count >= 3
                || pixel_color.r == player3_color.r
                    && pixel_color.g == player3_color.g
                    && pixel_color.b == player3_color.b
            {
                player3_count += 1;
            } else if players_count >= 4
                || pixel_color.r == player4_color.r
                    && pixel_color.g == player4_color.g
                    && pixel_color.b == player4_color.b
            {
                player4_count += 1;
            }
        }
    }
    [
        player1_count as f32
            / (player1_count + player2_count + player3_count + player4_count) as f32,
        player2_count as f32
            / (player1_count + player2_count + player3_count + player4_count) as f32,
        player3_count as f32
            / (player1_count + player2_count + player3_count + player4_count) as f32,
        player4_count as f32
            / (player1_count + player2_count + player3_count + player4_count) as f32,
    ]
}
