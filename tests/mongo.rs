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
    let mut theme= db.find_document::<Theme>(doc!(
        "name":"fall"
    ), None).unwrap().unwrap();
    assert_eq!("fall", &theme.name);
}
#[test]
fn find_user() {
    let db = connect();
    let mut user = db.find_document::<User>(doc!(
        "name":"nicohman"
    ), None).unwrap().unwrap();
    assert_eq!("nicohman", &user.name);
}
