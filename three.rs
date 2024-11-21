use crate::{model, screen};

// A struct that represents a 3D point in space with x, y, and z coordinates.
#[derive(Copy, Clone)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point {
    // Constructor to create a new Point given x, y, z coordinates.
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Point { x, y, z }
    }
}

// The Camera struct represents the camera's position and orientation in 3D space.
pub struct Camera {
    // Camera's position in world space
    pub coordinates: Point,

    // Rotation angles for yaw, pitch, and roll in radians.
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,

    // Distance from the camera to the viewport, affecting how 3D points are projected.
    pub viewport_distance: f32,

    // The camera's field of view (FOV) in radians.
    pub viewport_fov: f32,

    // A reference to the screen where the 3D model will be rendered.
    pub screen: screen::Screen,
}

impl Camera {
    // Creates a new Camera instance with specified parameters.
    pub fn new(
        coordinates: Point,
        yaw: f32,
        pitch: f32,
        roll: f32,
        viewport_distance: f32,
        viewport_fov: f32,
    ) -> Self {
        Camera {
            coordinates,
            yaw,
            pitch,
            roll,
            viewport_distance,
            viewport_fov,
            screen: screen::Screen::new(),
        }
    }

    // Converts a world coordinate (point) into camera space by applying the inverse of the camera's transformations (yaw, pitch, roll).
    fn world_to_camera(&self, world_point: &Point) -> Point {
        // Precompute sine and cosine of yaw, pitch, and roll for use in rotations.
        let (sin_yaw, sin_pitch, sin_roll) = (self.yaw.sin(), self.pitch.sin(), self.roll.sin());
        let (cos_yaw, cos_pitch, cos_roll) = (self.yaw.cos(), self.pitch.cos(), self.roll.cos());

        // Calculate the relative position of the world point from the camera's coordinates.
        let delta_x = world_point.x - self.coordinates.x;
        let delta_y = world_point.y - self.coordinates.y;
        let delta_z = world_point.z - self.coordinates.z;

        // Apply yaw rotation (around the z-axis) to align the point with the camera's orientation.
        let (rot_x, rot_y, rot_z) = (
            delta_x * cos_yaw - delta_z * sin_yaw,
            delta_y,
            delta_x * sin_yaw + delta_z * cos_yaw,
        );

        // Apply pitch rotation (around the y-axis) to adjust the point for the camera's tilt.
        let (pitch_x, pitch_y, pitch_z) = (
            rot_x,
            rot_y * cos_pitch - rot_z * sin_pitch,
            rot_y * sin_pitch + rot_z * cos_pitch,
        );

        // Apply roll rotation (around the x-axis) to finalize the point's transformation.
        let (roll_x, roll_y, roll_z) = (
            pitch_x * cos_roll - pitch_y * sin_roll,
            pitch_x * sin_roll + pitch_y * cos_roll,
            pitch_z,
        );

        Point::new(roll_x, roll_y, roll_z)
    }

    // Converts a 3D point in camera space to 2D screen coordinates for rendering.
    fn camera_to_screen(&self, camera_point: &Point) -> screen::Point {
        // Project the 3D point onto the 2D viewport.
        let projected_x = camera_point.x * self.viewport_distance / camera_point.z;
        let projected_y = camera_point.y * self.viewport_distance / camera_point.z;

        // Calculate the viewport dimensions based on the camera's FOV and the screen's aspect ratio.
        let viewport_width = 2.0 * self.viewport_distance * (self.viewport_fov / 2.0).tan();
        let viewport_height = (self.screen.height as f32 / self.screen.width as f32) * viewport_width;

        // Convert the projected coordinates into screen coordinates.
        let screen_x = (projected_x / viewport_width + 0.5) * self.screen.width as f32;
        let screen_y = (1.0 - (projected_y / viewport_height + 0.5)) * self.screen.height as f32;

        // Return the final screen coordinates rounded to the nearest pixel.
        screen::Point::new(screen_x.round() as i32, screen_y.round() as i32)
    }

    // Renders the points of a 3D model onto the screen.
    pub fn plot_model_points(&mut self, model: &model::Model) {
        for point in model.points.iter() {
            self.write(true, &model.model_to_world(point));
        }
    }

    // Renders the edges of a 3D model by connecting its points with lines.
    pub fn plot_model_edges(&mut self, model: &model::Model) {
        for edge in model.edges.iter() {
            self.edge(
                &model.model_to_world(&edge.0),
                &model.model_to_world(&edge.1),
            );
        }
    }

    // Renders a single 3D point by converting it to camera and then screen coordinates.
    pub fn write(&mut self, val: bool, point: &Point) {
        let camera_point = self.world_to_camera(point);
        if camera_point.z >= self.viewport_distance {
            self.screen.write(val, &self.camera_to_screen(&camera_point));
        }
    }

    // Renders an edge (a line) between two points, clipping if necessary.
    pub fn edge(&mut self, start: &Point, end: &Point) {
        // Convert both points to camera space.
        let camera_start = self.world_to_camera(start);
        let camera_end = self.world_to_camera(end);

        // Check if any point is behind the viewport and needs to be clipped.
        let clip_start = camera_start.z < self.viewport_distance;
        let clip_end = camera_end.z < self.viewport_distance;

        // If both points are behind the viewport, we do not render the edge.
        if clip_start && clip_end {
            return;
        }

        // If neither point is behind the viewport, draw the line between them.
        if !clip_start && !clip_end {
            self.screen.line(
                &self.camera_to_screen(&camera_start),
                &self.camera_to_screen(&camera_end),
            );
            return;
        }

        // If one point is behind the viewport, clip the line to the viewport.
        let (clipped, unclipped) = if clip_start {
            (camera_start, camera_end)
        } else {
            (camera_end, camera_start)
        };

        // Calculate the point where the clipped point intersects the viewport.
        let distance_to_clip = self.viewport_distance - clipped.z;
        let (delta_x, delta_y, delta_z) = (
            unclipped.x - clipped.x,
            unclipped.y - clipped.y,
            unclipped.z - clipped.z,
        );
        let lambda = distance_to_clip / delta_z;

        // Compute the new clipped point at the intersection.
        let new_clipped = Point::new(
            lambda * delta_x + clipped.x,
            lambda * delta_y + clipped.y,
            self.viewport_distance,
        );

        // Draw the clipped line from the new clipped point to the unclipped point.
        self.screen.line(
            &self.camera_to_screen(&new_clipped),
            &self.camera_to_screen(&unclipped),
        );
    }
}
