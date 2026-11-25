
use macroquad::prelude::*;

const W: usize = 10;
const H: usize = 20;
const CELL: f32 = 28.0;

const SHAPES: [[(i32, i32); 4]; 7] = [
    [(0,0),(1,0),(2,0),(3,0)],
    [(0,0),(1,0),(0,1),(1,1)],
    [(1,0),(0,1),(1,1),(2,1)],
    [(0,0),(0,1),(1,1),(2,1)],
    [(2,0),(0,1),(1,1),(2,1)],
    [(1,0),(2,0),(0,1),(1,1)],
    [(0,0),(1,0),(1,1),(2,1)],
];

fn window_conf() -> Conf {
    Conf {
        window_title: "Falling Blocks".to_owned(),
        window_width: 780,
        window_height: 700,
        window_resizable: false,
        ..Default::default()
    }
}

fn random_piece() -> usize {
    rand::gen_range(0, 7)
}

fn piece_color(piece: usize) -> Color {
    match piece {
        0 => Color::from_rgba(65, 160, 220, 255),
        1 => Color::from_rgba(235, 190, 75, 255),
        2 => Color::from_rgba(155, 110, 220, 255),
        3 => Color::from_rgba(75, 175, 125, 255),
        4 => Color::from_rgba(225, 120, 75, 255),
        5 => Color::from_rgba(220, 85, 100, 255),
        _ => Color::from_rgba(70, 190, 185, 255),
    }
}

fn cells(piece: usize, rotation: usize) -> [(i32, i32); 4] {
    let mut result = SHAPES[piece];

    for _ in 0..rotation {
        for cell in &mut result {
            let old_x = cell.0;
            cell.0 = -cell.1;
            cell.1 = old_x;
        }
    }

    let min_x = result.iter().map(|c| c.0).min().unwrap();
    let min_y = result.iter().map(|c| c.1).min().unwrap();

    for cell in &mut result {
        cell.0 -= min_x;
        cell.1 -= min_y;
    }

    result
}

fn valid(
    board: &[[Option<Color>; W]; H],
    piece: usize,
    rotation: usize,
    x: i32,
    y: i32,
) -> bool {
    for (cx, cy) in cells(piece, rotation) {
        let px = x + cx;
        let py = y + cy;

        if px < 0 || px >= W as i32 || py >= H as i32 {
            return false;
        }

        if py >= 0 && board[py as usize][px as usize].is_some() {
            return false;
        }
    }

    true
}

fn lock_piece(
    board: &mut [[Option<Color>; W]; H],
    piece: usize,
    rotation: usize,
    x: i32,
    y: i32,
    color: Color,
    score: &mut u32,
    lines: &mut u32,
    level: u32,
) -> bool {
    let shape = cells(piece, rotation);

    for (cx, cy) in shape {
        if y + cy < 0 {
            return true;
        }
    }

    for (cx, cy) in shape {
        board[(y + cy) as usize][(x + cx) as usize] = Some(color);
    }

    let mut new_board = [[None; W]; H];
    let mut write_row = H;
    let mut cleared = 0;

    for read_row in (0..H).rev() {
        if board[read_row].iter().all(|cell| cell.is_some()) {
            cleared += 1;
        } else {
            write_row -= 1;
            new_board[write_row] = board[read_row];
        }
    }

    *board = new_board;

    let points = match cleared {
        1 => 100,
        2 => 300,
        3 => 500,
        4 => 800,
        _ => 0,
    };

    *score += points * level;
    *lines += cleared as u32;

    false
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut board = [[None; W]; H];

    let mut piece = random_piece();
    let mut next_piece = random_piece();

    let mut rotation = 0;
    let mut x = 3;
    let mut y = 0;

    let mut score = 0;
    let mut lines = 0;
    let mut level = 1;

    let mut timer = 0.0;

    let mut started = false;
    let mut paused = false;
    let mut game_over = false;

    let bg = Color::from_rgba(25, 32, 43, 255);
    let panel = Color::from_rgba(35, 45, 59, 255);
    let grid = Color::from_rgba(48, 60, 76, 255);
    let text_color = Color::from_rgba(225, 232, 240, 255);
    let muted = Color::from_rgba(145, 160, 180, 255);

    let board_x = 250.0;
    let board_y = 65.0;
    let board_width = W as f32 * CELL;
    let board_height = H as f32 * CELL;

    let left_x = 35.0;
    let right_x = 560.0;

    loop {
        let dt = get_frame_time();

        if !started && is_key_pressed(KeyCode::Enter) {
            started = true;
        }

        if is_key_pressed(KeyCode::R) {
            board = [[None; W]; H];
            piece = random_piece();
            next_piece = random_piece();
            rotation = 0;
            x = 3;
            y = 0;
            score = 0;
            lines = 0;
            level = 1;
            timer = 0.0;
            started = true;
            paused = false;
            game_over = false;
        }

        if started && !game_over && is_key_pressed(KeyCode::P) {
            paused = !paused;
        }

        if started && !paused && !game_over {
            if is_key_pressed(KeyCode::Left)
                && valid(&board, piece, rotation, x - 1, y) {
                x -= 1;
            }

            if is_key_pressed(KeyCode::Right)
                && valid(&board, piece, rotation, x + 1, y) {
                x += 1;
            }

            if is_key_pressed(KeyCode::Up) {
                let next_rotation = (rotation + 1) % 4;

                if valid(&board, piece, next_rotation, x, y) {
                    rotation = next_rotation;
                }
            }

            let mut should_lock = false;

            if is_key_pressed(KeyCode::Space) {
                while valid(&board, piece, rotation, x, y + 1) {
                    y += 1;
                    score += 2;
                }

                should_lock = true;
            } else {
                timer += dt;

                let speed = (0.5 * 0.85_f32.powi((level - 1) as i32)).max(0.08);
                let interval = if is_key_down(KeyCode::Down) {
                    0.04
                } else {
                    speed
                };

                if timer >= interval {
                    timer = 0.0;

                    if valid(&board, piece, rotation, x, y + 1) {
                        y += 1;

                        if is_key_down(KeyCode::Down) {
                            score += 1;
                        }
                    } else {
                        should_lock = true;
                    }
                }
            }

            if should_lock {
                let top_out = lock_piece(
                    &mut board,
                    piece,
                    rotation,
                    x,
                    y,
                    piece_color(piece),
                    &mut score,
                    &mut lines,
                    level,
                );

                if top_out {
                    game_over = true;
                } else {
                    level = lines / 10 + 1;

                    piece = next_piece;
                    next_piece = random_piece();

                    rotation = 0;
                    x = 3;
                    y = 0;

                    if !valid(&board, piece, rotation, x, y) {
                        game_over = true;
                    }
                }
            }
        }

        clear_background(bg);

        // Header
        draw_text("FALLING BLOCKS", 35.0, 38.0, 27.0, text_color);
        draw_text("A CLASSIC BLOCK PUZZLE", 35.0, 57.0, 12.0, muted);

        // Side panels
        draw_rectangle(left_x, 85.0, 175.0, 230.0, panel);
        draw_rectangle(right_x, 85.0, 185.0, 205.0, panel);
        draw_rectangle(right_x, 310.0, 185.0, 210.0, panel);

        // Score panel
        draw_text("YOUR SCORE", left_x + 15.0, 120.0, 15.0, muted);
        draw_text(&format!("{}", score), left_x + 15.0, 160.0, 32.0, text_color);

        draw_line(left_x + 15.0, 180.0, left_x + 160.0, 180.0, 1.0, grid);

        draw_text("LEVEL", left_x + 15.0, 215.0, 14.0, muted);
        draw_text(&format!("{}", level), left_x + 15.0, 250.0, 25.0, text_color);

        draw_text("LINES", left_x + 90.0, 215.0, 14.0, muted);
        draw_text(&format!("{}", lines), left_x + 90.0, 250.0, 25.0, text_color);

        // Next piece panel
        draw_text("NEXT PIECE", right_x + 15.0, 120.0, 16.0, text_color);

        let preview = cells(next_piece, 0);
        let max_x = preview.iter().map(|c| c.0).max().unwrap() + 1;
        let max_y = preview.iter().map(|c| c.1).max().unwrap() + 1;

        let preview_cell = 23.0;
        let start_x = right_x + (185.0 - max_x as f32 * preview_cell) / 2.0;
        let start_y = 155.0 + (90.0 - max_y as f32 * preview_cell) / 2.0;

        for (cx, cy) in preview {
            draw_rectangle(
                start_x + cx as f32 * preview_cell,
                start_y + cy as f32 * preview_cell,
                preview_cell - 2.0,
                preview_cell - 2.0,
                piece_color(next_piece),
            );
        }

        // Controls panel
        draw_text("CONTROLS", right_x + 15.0, 345.0, 16.0, text_color);

        let controls = [
            ("LEFT / RIGHT", "Move"),
            ("UP", "Rotate"),
            ("DOWN", "Soft drop"),
            ("SPACE", "Hard drop"),
            ("P", "Pause"),
            ("R", "Restart"),
        ];

        for (i, (key, action)) in controls.iter().enumerate() {
            let py = 380.0 + i as f32 * 23.0;
            draw_text(key, right_x + 15.0, py, 12.0, muted);
            draw_text(action, right_x + 110.0, py, 12.0, text_color);
        }

        // Game board
        draw_rectangle(
            board_x - 5.0,
            board_y - 5.0,
            board_width + 10.0,
            board_height + 10.0,
            panel,
        );

        for row in 0..H {
            for col in 0..W {
                let px = board_x + col as f32 * CELL;
                let py = board_y + row as f32 * CELL;

                draw_rectangle(px, py, CELL, CELL, grid);

                if let Some(block_color) = board[row][col] {
                    draw_rectangle(
                        px + 1.0,
                        py + 1.0,
                        CELL - 2.0,
                        CELL - 2.0,
                        block_color,
                    );
                } else {
                    draw_rectangle(
                        px + 1.0,
                        py + 1.0,
                        CELL - 2.0,
                        CELL - 2.0,
                        Color::from_rgba(29, 38, 51, 255),
                    );
                }
            }
        }

        // Current falling piece
        if started && !game_over {
            for (cx, cy) in cells(piece, rotation) {
                let px = board_x + (x + cx) as f32 * CELL;
                let py = board_y + (y + cy) as f32 * CELL;

                draw_rectangle(
                    px + 1.0,
                    py + 1.0,
                    CELL - 2.0,
                    CELL - 2.0,
                    piece_color(piece),
                );
            }
        }

        // Start, pause and game-over overlays
        if !started || paused || game_over {
            draw_rectangle(
                board_x,
                board_y + 200.0,
                board_width,
                130.0,
                Color::from_rgba(15, 22, 32, 235),
            );

            let (heading, instruction) = if !started {
                ("FALLING BLOCKS", "PRESS ENTER TO START")
            } else if game_over {
                ("GAME OVER", "PRESS R TO PLAY AGAIN")
            } else {
                ("PAUSED", "PRESS P TO RESUME")
            };

            let heading_size = 27.0;
            let heading_width = measure_text(heading, None, heading_size as u16, 1.0).width;

            draw_text(
                heading,
                board_x + (board_width - heading_width) / 2.0,
                board_y + 255.0,
                heading_size,
                text_color,
            );

            let instruction_size = 15.0;
            let instruction_width =
                measure_text(instruction, None, instruction_size as u16, 1.0).width;

            draw_text(
                instruction,
                board_x + (board_width - instruction_width) / 2.0,
                board_y + 290.0,
                instruction_size,
                muted,
            );
        }

        draw_text(
            "Clear rows. Build your score. Keep going.",
            250.0,
            665.0,
            15.0,
            muted,
        );

        next_frame().await;
    }
}
