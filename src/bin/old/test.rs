fn main() {
    println!("Hello, world!");

    let x = 27;
    let forward = false;

    let x = if forward { 42 } else { x };

    println!("x: {}", x);
}
