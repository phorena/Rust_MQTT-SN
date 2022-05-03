use getset::{CopyGetters, Getters, Setters};

#[derive(Debug, Getters, Setters, CopyGetters)]
pub struct Foo
{
    pri: u32,
    
}
impl Default for Foo {
    fn default() -> Foo {
        Foo {
            pri: 17,
        }
    }
}

