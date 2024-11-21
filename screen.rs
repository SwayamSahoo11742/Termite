use std::*;
use crossterm::{
    execute, 
    terminal,
    cursor,
    style
};

// Default terminal dimensions in case querying the terminal size fails.
const DEFAULT_TERMINAL_DIMENSIONS: (u16, u16) = (80, 24);

// A trait for objects that have defined width and height dimensions.
pub trait Dim {
    const WIDTH: usize;
    const HEIGHT: usize;
}

// Implementation of the Dim trait for a 2D array of booleans with a constant width and height.
impl<const WIDTH: usize, const HEIGHT: usize> Dim for [[bool; WIDTH]; HEIGHT] {
    const WIDTH: usize = WIDTH;
    const HEIGHT: usize = HEIGHT;
}

// Trait for a "Pixel", which requires a way to create a new pixel and convert it to a character.
pub trait Pixel: Dim + ops::IndexMut<usize, Output=[bool; 2]> + Clone {
    fn new() -> Self;
    fn to_char(&self) -> char;
}

// A BlockPixel is a 2x2 grid of booleans representing a pixel.
pub type BlockPixel = [[bool; 2]; 2];
impl Pixel for BlockPixel {
    // Creates a new BlockPixel with all values set to false (off).
    fn new() -> BlockPixel { [[false; BlockPixel::WIDTH]; BlockPixel::HEIGHT] }
    
    // Converts the 2x2 grid of booleans into a Unicode character for rendering.
    fn to_char(&self) -> char {
        match self {
            // Define all possible 2x2 grid states and their corresponding characters.
            [[false, false], [false, false]] => ' ',
            [[true, false], [false, false]] => '▘',
            [[false, true], [false, false]] => '▝',
            [[true, true], [false, false]] => '▀',
            [[false, false], [true, false]] => '▖',
            [[true, false], [true, false]] => '▌',
            [[false, true], [true, false]] => '▞',
            [[true, true], [true, false]] => '▛',
            [[false, false], [false, true]] => '▗',
            [[true, false], [false, true]] => '▚',
            [[false, true], [false, true]] => '▐',
            [[true, true], [false, true]] => '▜',
            [[false, false], [true, true]] => '▄',
            [[true, false], [true, true]] => '▙',
            [[false, true], [true, true]] => '▟',
            [[true, true], [true, true]] => '█'
        }
    }
}

// A BraillePixel is a 4x2 grid representing Braille characters.
pub type BrailePixel = [[bool; 2]; 4];
impl Pixel for BrailePixel {
    // Creates a new BraillePixel with all values set to false (off).
    fn new() -> BrailePixel { [[false; BrailePixel::WIDTH]; BrailePixel::HEIGHT] }
    
    // Converts the 4x2 grid of booleans into a Unicode character for rendering as a Braille cell.
    fn to_char(&self) -> char {
        let mut unicode: u32 = 0;

        // Set bits for the 4x2 grid, mapping true values to the corresponding Braille character bits.
        if self[0][0] { unicode |= 1 << 0 }
        if self[1][0] { unicode |= 1 << 1 }
        if self[2][0] { unicode |= 1 << 2 }
    
        if self[0][1] { unicode |= 1 << 3 }
        if self[1][1] { unicode |= 1 << 4 }
        if self[2][1] { unicode |= 1 << 5 }
    
        if self[3][0] { unicode |= 1 << 6 }
        if self[3][1] { unicode |= 1 << 7 }
    
        unicode |= 0x28 << 8; // This is the Unicode range for Braille characters.
    
        // Convert the calculated bits into the corresponding Braille character.
        char::from_u32(unicode).unwrap()
    }
}

// A simple 2D point structure for x and y coordinates.
#[derive(Copy, Clone)]
pub struct Point {
    pub x: i32,
    pub y: i32
}

impl Point {
    // Constructor to create a new point with given x and y coordinates.
    pub fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }
}

// A struct to represent the screen with pixel data.
pub struct Screen {
    pub width: u16,
    pub height: u16,
    content: Vec<Vec<bool>>, // The screen's pixel content as a 2D array of booleans.
}

impl Screen {
    // Constructor to create a new screen, clearing the terminal and resetting the cursor to (0,0).
    pub fn new() -> Screen {
        execute!(
            io::stdout(),
            cursor::MoveTo(0, 0),
            terminal::Clear(terminal::ClearType::All)
        ).unwrap();

        Screen{
            content: Vec::new(),
            width: 0,
            height: 0
        }
    }

    // Resize the screen to fit the terminal size and a specified pixel type (e.g., BlockPixel).
    pub fn fit_to_terminal<T: Pixel>(&mut self) {
        let (terminal_width, terminal_height) = match terminal::size() {
            Ok(dim) => dim,
            Err(_) => DEFAULT_TERMINAL_DIMENSIONS // Use default if terminal size is unavailable.
        };

        // Resize screen based on the terminal size and the dimensions of the pixel type.
        self.resize(
            terminal_width * T::WIDTH as u16, 
            (terminal_height - 1) * T::HEIGHT as u16
        );
    }

    // Write a value to a specific coordinate on the screen, if within bounds.
    pub fn write(&mut self, val: bool, point: &Point) {
        let x_in_bounds = 0 < point.x && point.x < self.width as i32;
        let y_in_bounds = 0 < point.y && point.y < self.height as i32;
        if x_in_bounds && y_in_bounds {
            self.content[point.y as usize][point.x as usize] = val;
        }
    }

    // Clears the entire screen by resetting the content to false (off).
    pub fn clear(&mut self) {
        self.content = vec![vec![false; self.width as usize]; self.height as usize];
    }

    // Resize the screen to a new width and height, adjusting content if necessary.
    pub fn resize(&mut self, width: u16, height: u16) {
        // Handle resizing the height.
        if height > self.height {
            self.content.extend(vec![
                vec![false; width as usize]; 
                (height - self.height) as usize
            ])
        } else {
            self.content.truncate(height as usize);
        }
        self.height = height;

        // Handle resizing the width.
        if width > self.width {
            for row in self.content.iter_mut() {
                row.extend(vec![false; (width - self.width) as usize]);
            }
        } else {
            for row in self.content.iter_mut() {
                row.truncate(width as usize);
            }
        }
        self.width = width;
    }

    // Draw a line on the screen using Bresenham's line algorithm.
    pub fn line(&mut self, start: &Point, end: &Point) {            
        let delta_x = (end.x - start.x).abs();
        let step_x: i32 = if start.x < end.x {1} else {-1};
        let delta_y = -(end.y - start.y).abs();
        let step_y: i32 = if start.y < end.y {1} else {-1};
        let mut err = delta_x + delta_y;

        let mut x = start.x;
        let mut y = start.y;

        self.write(true, &Point::new(x, y)); // Draw the starting point.

        // Loop until the end point is reached.
        while !(x == end.x && y == end.y) {
            self.write(true, &Point::new(x, y)); // Draw the current point.
            let curr_err = err;

            if 2 * curr_err >= delta_y {
                err += delta_y;
                x += step_x;
            }

            if 2 * curr_err <= delta_x {
                err += delta_x;
                y += step_y;
            }
        }
    }

    // Render the screen by outputting its content using the specified pixel type.
    pub fn render<PixelType: Pixel>(&self) {
        execute!(
            io::stdout(),
            cursor::MoveTo(0, 0) // Move cursor to the top-left corner.
        ).unwrap();

        // Split the content into chunks according to the height of the pixel type.
        let chunked_rows = self.content.chunks(PixelType::HEIGHT);

        // Iterate through each chunked row and render the appropriate characters.
        for subrows in chunked_rows {
            let real_row_width = self.width.div_ceil(PixelType::WIDTH as u16) as usize;
            let mut real_row = vec![PixelType::new(); real_row_width];

            // Convert booleans into pixels and update the content for rendering.
            for y in 0..PixelType::HEIGHT {
                for x in 0..real_row_width {
                    let pix = &mut real_row[x];
                    for j in 0..PixelType::WIDTH {
                        pix[y][j] = subrows[y as usize][x + j];
                    }
                }
            }

            // Output the resulting character representation for the row.
            for pixel in real_row {
                print!("{}", pixel.to_char());
            }
        }
    }
}
