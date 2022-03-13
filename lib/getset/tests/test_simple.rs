use crate::simple::Foo;
mod simple;

    fn constraint_private(_val: u32) -> bool {
        true
    }

#[test]
fn test(){
    let mut foo = Foo::default();
    foo.set_pri(2);
    // (*foo.private_mut()) += 1;
    assert_eq!(*foo.pri(), 2);
    // assert_eq!(foo.private, 2);
    // foo.private = 92;
    foo.set_private(92);
    assert_eq!(*foo.pri(), 92);
    dbg!(&foo);
    println!("***** {:?}", &foo);
    let bar = Foo::default();
}
