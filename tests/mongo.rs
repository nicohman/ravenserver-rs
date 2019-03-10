extern crate ravenserver;
#[macro_use]
extern crate mongodb;
use ravenserver::mongo::*;
use ravenserver::themes::*;
#[test]
fn find_theme() {
    let themes = DataBase::new("127.0.0.1",27017).expect("Couldn't connect to mongo server");
    let mut docs = themes.find_documents::<Theme>(doc!(
        "name":"fall"
    ), None).unwrap();
    let theme = docs.remove(0).unwrap();
    assert_eq!("fall", &theme.name);
}
