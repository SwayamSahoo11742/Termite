use crate::three;
use std::*;

// Error struct for parsing .obj file failures.
#[derive(Debug)]
struct ObjParseError;

// Implementing methods for ObjParseError.
impl ObjParseError {
    // Constructor for ObjParseError.
    fn new() -> ObjParseError {
        ObjParseError
    }
}

// Implementing Display trait to define how ObjParseError should be printed.
impl fmt::Display for ObjParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error parsing .obj file.")
    }
}

// Implementing the Error trait for ObjParseError to allow it to be used as an error type.
impl error::Error for ObjParseError {
    fn description(&self) -> &str {
        "Error parsing .obj file."
    }
}

// Struct representing a 3D model.
pub struct Model {
    // List of points (vertices) defined in model space.
    pub points: Vec<three::Point>,
    // List of edges, each represented as a tuple of points (start and end).
    pub edges: Vec<(three::Point, three::Point)>,

    // Position of the model in world space (corresponds to the (0, 0, 0) point in model space).
    pub position: three::Point,
}

#[allow(dead_code)]
impl Model {
    // Constructor for creating a new model with the specified points, edges, and position.
    pub fn new(
        points: Vec<three::Point>,
        edges: Vec<(three::Point, three::Point)>,
        position: three::Point,
    ) -> Model {
        Model {
            points,
            position,
            edges,
        }
    }

    // Creates a new cube model with a specified side length, centered at a specified position.
    pub fn new_cube(side_length: f32, position: three::Point) -> Model {
        // Define the four corners of the front face of the cube.
        let front = (
            three::Point::new(-side_length / 2., -side_length / 2., side_length / 2.),
            three::Point::new(-side_length / 2., side_length / 2., side_length / 2.),
            three::Point::new(side_length / 2., side_length / 2., side_length / 2.),
            three::Point::new(side_length / 2., -side_length / 2., side_length / 2.),
        );

        // Define the four corners of the rear face of the cube.
        let rear = (
            three::Point::new(-side_length / 2., -side_length / 2., -side_length / 2.),
            three::Point::new(-side_length / 2., side_length / 2., -side_length / 2.),
            three::Point::new(side_length / 2., side_length / 2., -side_length / 2.),
            three::Point::new(side_length / 2., -side_length / 2., -side_length / 2.),
        );

        // Return a new Model instance with the cube's edges.
        Model {
            points: Vec::new(), // Empty points since we only define edges for the cube here.
            edges: vec![
                // Front face edges.
                (front.0, front.1),
                (front.1, front.2),
                (front.2, front.3),
                (front.3, front.0),

                // Rear face edges.
                (rear.0, rear.1),
                (rear.1, rear.2),
                (rear.2, rear.3),
                (rear.3, rear.0),

                // Edges connecting front and rear faces.
                (rear.0, front.0),
                (rear.1, front.1),
                (rear.2, front.2),
                (rear.3, front.3),
            ],
            position,
        }
    }

    // Creates a model from a .obj file, placing it at a specified position in world space.
    pub fn new_obj(path: &str, position: three::Point) -> Result<Model, Box<dyn error::Error>> {
        // Read the contents of the .obj file into a string.
        let mut code = fs::read_to_string(path)?;

        // Pre-process the code to handle escaped newlines that continue to the next line.
        code = code.replace("\\\n", " ");
        
        // Vectors to store parsed vertices, lines, and faces.
        let mut vertices = Vec::<three::Point>::new();
        let mut lines = Vec::<Vec<usize>>::new();
        let mut faces = Vec::<Vec<usize>>::new();

        // Iterate through each line in the .obj file.
        for line in code.split('\n') {
            // Split the line into tokens (words or numbers).
            let mut tokens = line.split_whitespace().filter(|&line| !line.is_empty());

            // Process the command at the start of the line.
            match tokens.next() {
                // Handle vertex definitions ("v").
                Some("v") => {
                    match (tokens.next(), tokens.next(), tokens.next(), tokens.next(), tokens.next()) {
                        (Some(x), Some(y), Some(z), _, None) => {
                            // Parse the x, y, z coordinates and store the vertex.
                            let x = x.parse::<f32>()?;
                            let y = y.parse::<f32>()?;
                            let z = z.parse::<f32>()?;
                            vertices.push(three::Point::new(x, y, z));
                        }
                        _ => {
                            // If the line format is invalid, return a parsing error.
                            return Err(Box::from(ObjParseError::new()))
                        }
                    }
                }

                // Handle line definitions ("l").
                Some("l") => {
                    let mut line = Vec::<usize>::new();
                    for point in tokens {
                        // Each point is given as an index, so split it by slashes (if present).
                        let mut params = point.split('/');
                        
                        // Parse the vertex index and add it to the line.
                        match (params.next(), params.next(), params.next()) {
                            (Some(vertex_index), _, None) => {
                                let vertex_index = vertex_index.parse::<usize>()?;
                                let vertex_index = vertex_index.checked_sub(1)?;
                                line.push(vertex_index);
                            }
                            _ => {
                                // Invalid line format.
                                return Err(Box::from(ObjParseError::new()))
                            }
                        }
                    }

                    // Add the line to the lines vector.
                    lines.push(line);
                }

                // Handle face definitions ("f" or "fo").
                Some("f") | Some("fo") => {
                    let mut face = Vec::<usize>::new();
                    for point in tokens {
                        // Each point in a face refers to a vertex index.
                        let mut params = point.split('/');
                        
                        // Parse the vertex index and add it to the face.
                        match (params.next(), params.next(), params.next(), params.next()) {
                            (Some(vertex_index), _, _, None) => {
                                let vertex_index = vertex_index.parse::<usize>()?;
                                let vertex_index = vertex_index.checked_sub(1)?;
                                face.push(vertex_index);
                            }
                            _ => {
                                // Invalid face format.
                                return Err(Box::from(ObjParseError::new()))
                            }
                        }
                    }

                    // Add the face to the faces vector.
                    faces.push(face);
                }

                // Handle comments (lines starting with "#").
                Some("#") => {}

                // Skip any other unsupported lines.
                _ => {}
            }
        }

        // Convert the parsed lines and faces into edges (pairs of vertex indices).
        let mut edges = Vec::<(usize, usize)>::new();
        for line in lines.iter() {
            if line.len() >= 2 {
                for start in 0..line.len() - 1 {
                    let end = start + 1;
                    edges.push((line[start], line[end]));
                }
            }
        }
        for face in faces.iter() {
            if face.len() >= 2 {
                for start in 0..face.len() - 1 {
                    let end = start + 1;
                    edges.push((face[start], face[end]));
                }
                // Add the closing edge for the face.
                edges.push((face.last().unwrap(), face.first().unwrap()));
            }
        }

        // Remove duplicate edges for performance.
        edges.sort();
        edges.dedup();

        // Convert the edges from indices to actual points.
        let edges: Vec<(three::Point, three::Point)> = edges
            .into_iter()
            .map(|(start_index, end_index)| (vertices[start_index], vertices[end_index]))
            .collect();

        // Return the model with the parsed vertices, edges, and position.
        Ok(Model {
            points: vertices,
            edges,
            position,
        })
    }

    // Transforms a point from model space to world space based on the model's position.
    pub fn model_to_world(&self, point: &three::Point) -> three::Point {
        three::Point {
            x: point.x + self.position.x,
            y: point.y + self.position.y,
            z: point.z + self.position.z,
        }
    }

    // Returns the axis-aligned bounding box (AABB) of the model in world space.
    pub fn world_bounds(&self) -> (three::Point, three::Point) {
        // If the model has no points or edges, return a degenerate bounding box.
        if self.points.is_empty() && self.edges.is_empty() {
            return (three::Point::new(0., 0., 0.), three::Point::new(0., 0., 0.));
        }

        // Initialize min and max bounds with the position of the first point in world space.
        let mut min = self.model_to_world(&self.points[0]);
        let mut max = min.clone();

        // Iterate through all points in the model.
        for point in &self.points {
            let point = self.model_to_world(point);

            // Update the min and max bounds.
            if point.x < min.x {
                min.x = point.x;
            }
            if point.y < min.y {
                min.y = point.y;
            }
            if point.z < min.z {
                min.z = point.z;
            }
            if point.x > max.x {
                max.x = point.x;
            }
            if point.y > max.y {
                max.y = point.y;
            }
            if point.z > max.z {
                max.z = point.z;
            }
        }

        // Iterate through all edges and include their points in the bounding box.
        for (start, end) in &self.edges {
            let start = self.model_to_world(start);
            let end = self.model_to_world(end);

            // Update bounds for the start and end points of the edge.
            if start.x < min.x {
                min.x = start.x;
            }
            if start.y < min.y {
                min.y = start.y;
            }
            if start.z < min.z {
                min.z = start.z;
            }
            if start.x > max.x {
                max.x = start.x;
            }
            if start.y > max.y {
                max.y = start.y;
            }
            if start.z > max.z {
                max.z = start.z;
            }

            if end.x < min.x {
                min.x = end.x;
            }
            if end.y < min.y {
                min.y = end.y;
            }
            if end.z < min.z {
                min.z = end.z;
            }
            if end.x > max.x {
                max.x = end.x;
            }
            if end.y > max.y {
                max.y = end.y;
            }
            if end.z > max.z {
                max.z = end.z;
            }
        }

        // Return the bounding box as two points (min and max).
        (min, max)
    }
}
