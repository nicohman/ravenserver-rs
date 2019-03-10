extern crate ravenserver;
#[macro_use]
extern crate mongodb;
use ravenserver::mongo::*;
use ravenserver::themes::*;
fn connect() -> DataBase {
    let themes = DataBase::new("127.0.0.1",27017, "themes").expect("Couldn't connect to mongo server");
    themes
}
#[test]
fn find_theme() {
    let db = connect();
    let mut docs = db.find_documents::<Theme>(doc!(
        "name":"fall"
    ), None).unwrap();
    let theme = docs.remove(0).unwrap();
    assert_eq!("fall", &theme.name);
}
#[test]
fn find_user() {
    let db = connect();
    let mut docs = db.find_documents::<User>(doc!(
        "name":"nicohman"
    ), None).unwrap();
    let user = docs.remove(0).unwrap();
    assert_eq!("nicohman", &user.name);
}
