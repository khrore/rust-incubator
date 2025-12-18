#[derive(Copy, Clone, Default, Debug)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Clone, Debug)]
struct Polyline {
    points: Vec<Point>,
}

impl Polyline {
    fn new(points: Vec<Point>) -> Option<Self> {
        if points.is_empty() {
            None
        } else {
            Some(Polyline { points })
        }
    }

    fn from_points(points: &[Point]) -> Option<Self> {
        if points.is_empty() {
            None
        } else {
            Some(Polyline {
                points: points.to_vec(),
            })
        }
    }
}

fn main() {
    // Demonstrate Point with Copy and Default
    println!("=== Point (Copy + Default) ===");

    // Default point (0, 0)
    let default_point = Point::default();
    println!("Default point: {:?}", default_point);

    // Create a point
    let p1 = Point { x: 10, y: 20 };
    println!("Point p1: {:?}", p1);

    // Copy semantics - p1 is copied implicitly
    let p2 = p1; // This is a copy, not a move
    println!("Point p2 (copy of p1): {:?}", p2);
    println!("p1 is still accessible: {:?}", p1); // p1 still exists!

    // Demonstrate Polyline with Clone (not Default)
    println!("\n=== Polyline (Clone, not Default) ===");

    // Create some points and demonstrate field usage
    let points = vec![
        Point { x: 0, y: 0 },
        Point { x: 10, y: 0 },
        Point { x: 10, y: 10 },
        Point { x: 0, y: 10 },
    ];

    // Calculate distance between first and last point to use the fields
    let start = points[0];
    let end = points[points.len() - 1];
    let distance = ((end.x - start.x).pow(2) + (end.y - start.y).pow(2)) as f64;
    println!("Distance from start to end point: {:.2}", distance);

    // Create polyline using the 'new' constructor
    let polyline = Polyline::new(points).expect("Polyline must have at least one point");
    println!("Polyline created with {} points", polyline.points.len());

    // Clone semantics - explicit cloning required
    let polyline_clone = polyline.clone(); // Explicit clone needed
    println!("Cloned polyline has {} points", polyline_clone.points.len());

    // Demonstrate move semantics for Polyline
    let another_polyline = Polyline::from_points(&[Point { x: 1, y: 1 }]).unwrap();
    println!("Created another polyline");

    // This would move the value if we assigned it
    let moved_polyline = another_polyline;
    // println!("{:?}", another_polyline); // This would fail - value was moved

    println!("Moved polyline has {} points", moved_polyline.points.len());
}
