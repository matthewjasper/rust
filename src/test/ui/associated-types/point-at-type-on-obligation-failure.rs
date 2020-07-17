trait Bar {
    type Ok;
    type Sibling: Bar2<Ok = Self::Ok>;
}
trait Bar2 {
    type Ok;
}

struct Foo;
struct Foo2;

impl Bar for Foo {
    type Ok = ();
    type Sibling = Foo2;
    //~^ ERROR type mismatch resolving `<Foo as Bar>::Ok == u32`
}
impl Bar2 for Foo2 {
    type Ok = u32;
}

fn main() {}
