use crossterm::{cursor::MoveTo, style::Print, ExecutableCommand};
use std::io::{self, stdout};

pub fn convert_to_bool_array(input: [[i32; 10]; 10]) -> [[bool; 10]; 10] {
    let mut bool_array = [[false; 10]; 10];

    for i in 0..10 {
        for j in 0..10 {
            bool_array[i][j] = input[i][j] != 0;
        }
    }

    bool_array
}

pub fn parse_board_coordinates(input: &str) -> Result<(u8, u8), String> {
    if input.len() != 2 {
        return Err("Invalid input length".to_string());
    }

    let x_char = input.chars().next().ok_or("Invalid input")?;
    let y_char = input.chars().nth(1).ok_or("Invalid input")?;

    let x_offset = b'A';
    let y_offset = b'0';

    let x_val = x_char as u8;
    let y_val = y_char as u8;

    if x_val < b'A' || y_val < b'0' {
        return Err("Invalid board coordinates".to_string());
    }

    let x = x_val - x_offset;
    let y = y_val - y_offset;

    if x > 9 || y > 9 {
        return Err("Invalid board coordinates".to_string());
    }

    Ok((x, y))
}

pub fn draw_board<T: Copy>(board: [[T; 10]; 10], position: (u16, u16)) -> io::Result<()>
where
    char: From<T>,
{
    let (mut x, mut y) = (position.0, position.1);

    stdout().execute(MoveTo(x, y))?;
    for i in 0..10 {
        x += 2;
        stdout().execute(MoveTo(x, y))?.execute(Print(i))?;
    }

    x = position.0;
    for i in 0..10 {
        y += 1;
        stdout()
            .execute(MoveTo(x, y))?
            .execute(Print((i + 65) as u8 as char))?;
    }

    for (y_1, row) in board.iter().enumerate() {
        for (x_1, field) in row.iter().enumerate() {
            y = position.1 + y_1 as u16 + 1;
            x = position.0 + (x_1 * 2) as u16 + 2;
            stdout()
                .execute(MoveTo(x, y))?
                .execute(Print::<char>((*field).into()))?;
        }
    }

    Ok(())
}
