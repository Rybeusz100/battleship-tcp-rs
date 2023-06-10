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
