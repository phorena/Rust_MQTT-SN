use getset::{CopyGetters, Getters, MutGetters, Setters};

#[derive(Debug, Getters, Setters, MutGetters, CopyGetters)]
pub struct Foo
{
    #[getset(get = "pub", set = "pub")]
    pri: u32,
}
impl Default for Foo {
    fn default() -> Foo {
        Foo {
            pri: 17,
        }
    }
}

    fn constraint_private(_val: u32) -> bool {
        true
    }

