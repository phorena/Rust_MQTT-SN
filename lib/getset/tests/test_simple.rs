use crate::simple::Foo;
mod simple;

    fn constraint_private(_val: u32) -> bool {
        true
    }

#[test]
fn test(){
    let mut foo = Foo::default();
    // (*foo.private_mut()) += 1;
    // assert_eq!(foo.private, 2);
    // foo.private = 92;
    dbg!(&foo);
    println!("***** {:?}", &foo);
    let bar = Foo::default();
}
