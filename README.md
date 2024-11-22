# Termite



![term](https://github.com/user-attachments/assets/e8aef4b5-8b82-4831-92cd-42be10c8a4fb)




Termite is a terminal based 3d renderer

# Demo
[Video Link](https://youtu.be/NClyZ4eBxyg)


# Purpose

- 3d rendering on the terminal
- Fun
- School Hackathon
- ???
- wanted to do rust and settleback on ui

# Image Gallery
![ScreenRecording2024-11-21191737-ezgif com-video-to-gif-converter](https://github.com/user-attachments/assets/723bea29-914b-4cd5-b280-cde9590edccc)



# Tech Stack and Tools
- Rust
- Python


## Process

### 1. **Setup and Dependencies**
This project leverages the Rust programming language with the following dependencies:
- `crossterm` for terminal manipulation (cursor control, text rendering).
- Standard library modules (`std::*`) for I/O, mathematical operations, and trait definitions.

### 2. **Pixel Representation**
The core of the project involves rendering a 3D model in a terminal by projecting points onto a 2D screen. To simulate pixel behavior, two types of pixel representations are defined:
- **Block Pixels:** 2x2 boolean grids mapped to Unicode characters for coarse resolution.
- **Braille Pixels:** 4x2 grids representing Braille characters for finer resolution.

#### Mathematical Encoding of Pixels:
Each grid configuration is converted into a corresponding Unicode character. For instance:
- A 2x2 grid has 16 possible states (\(2^4\) combinations), represented by the characters `' '`, `'▘'`, `'▝'`, ..., `'█'`.
- For Braille pixels (4x2 grid), the Unicode value is calculated by:
  
  $$\text{Unicode} = 0x2800 + \sum_{i=0}^{7} b_i \times 2^i$$
  
  where $$\( b_i \)$$ is the boolean value of each grid cell (true/false), and `0x2800` is the starting point of Braille characters in Unicode.

### 3. **Screen Management**
The `Screen` struct manages terminal dimensions and content:
- **Resizing Mechanism:** Ensures the screen dynamically adjusts to terminal size:
 
  $$\text{Screen Width (pixels)} = \text{Terminal Width} \times \text{Pixel Width}$$
  
  
  $$\text{Screen Height (pixels)} = (\text{Terminal Height} - 1) \times \text{Pixel Height}$$
  

- **Drawing Lines:** Uses Bresenham's Line Algorithm for rasterizing lines between two points $$(x_1, y_1)$$ and $$(x_2, y_2)$$:
  
  $$\text{Error term: } \varepsilon = \Delta x + \Delta y$$
  
  The algorithm iterates through the pixels, updating $$\( x \)$$ and $$\( y \)$$ based on the error term:
  
  $$\text{If } 2\varepsilon \geq \Delta y, \quad x += \text{step}_x, \quad \varepsilon += \Delta y$$
  
  $$\text{If } 2\varepsilon \leq \Delta x, \quad y += \text{step}_y, \quad \varepsilon += \Delta x$$
  

### 4. **3D Camera and Projection**
The `Camera` struct handles the transformation from 3D world coordinates to 2D screen coordinates:
- **World to Camera Transformation:** Applies yaw, pitch, and roll rotations to map a world point $$(x_w, y_w, z_w)$$ to camera space $$(x_c, y_c, z_c)$$:

```math
  \begin{aligned}
  x' &= x_w \cos(\text{yaw}) - z_w \sin(\text{yaw}) \\
  y' &= y_w \cos(\text{pitch}) - z_w \sin(\text{pitch}) \\
  z' &= x_w \sin(\text{yaw}) + z_w \cos(\text{yaw}) \\
  \end{aligned}
```

- **Camera to Screen Projection:** Projects a 3D point onto a 2D plane using a perspective projection:

 $$ x_s = \left(\frac{x_c \times d}{z_c}\right), \quad y_s = \left(\frac{y_c \times d}{z_c}\right)$$

  where $$\( d \)$$ is the viewport distance.

- **Viewport Mapping:** Maps the projected 2D coordinates to screen pixels:
  
  $$x_{\text{screen}} = \left(\frac{x_s}{\text{viewport width}} + 0.5\right) \times \text{screen width}$$
  
  
  $$y_{\text{screen}} = \left(1 - \left(\frac{y_s}{\text{viewport height}} + 0.5\right)\right) \times \text{screen height}$$

  
### 5. **Model Rendering**
The rendering process involves:
- **Point Rendering:** Converting each 3D model point to screen coordinates.
- **Edge Rendering:** Drawing lines between pairs of points defined as edges.

### 6. **Final Rendering Pipeline**
1. Clear the terminal screen.
2. Fit the screen dimensions based on terminal size.
3. Project 3D points into 2D space.
4. Draw edges using Bresenham's algorithm.
5. Render the output using Unicode block or Braille characters.



