#[derive(Clone, Copy, Default, Debug)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Clone, Debug)]
struct Polyline {
    point: Point,
    next: Option<Box<Polyline>>,
}

fn main() {
    let p1 = Point::default();
    let p2 = p1;
    println!("Default point: {:?}", p1);
}
