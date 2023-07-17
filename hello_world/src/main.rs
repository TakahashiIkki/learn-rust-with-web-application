mod foo;
mod sum;

fn main() {
    foo::bar::baz();
    println!("{}", sum::add(1,2));
    println!("{}", sum::sub(10,2));
}
