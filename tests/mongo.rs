extern crate ravenserver;
#[macro_use]
extern crate mongodb;
use ravenserver::mongo::*;
use ravenserver::themes::*;
fn connect() -> DataBase {
    let themes =
        DataBase::new("127.0.0.1", 27017, "themes").expect("Couldn't connect to mongo server");
    themes
}
#[test]
fn find_theme() {
    let db = connect();
    let theme = db
        .find_one::<Theme>(
            doc!{
            "name":"fall"
                },
            None,
        )
        .unwrap()
        .unwrap();
    assert_eq!("fall", &theme.name);
}
#[test]
fn find_user() {
    let db = connect();
    let user = db
        .find_one::<User>(
            doc!{
            "name":"nicohman"
                },
            None,
        )
        .unwrap()
        .unwrap();
    assert_eq!("nicohman", &user.name);
}
#[test]
fn find_key_value() {
    let db = connect();
    let user: User = db.find_one_key_value("name", "nicohman").unwrap().unwrap();
    assert_eq!("nicohman", &user.name);
}
#[test]
fn save() {
    let db = connect();
    let mut theme : Theme = db.find_one_key_value("name", "fall").unwrap().unwrap();
    theme.name = "autumn".to_string();
    let old_screen = theme.screen.clone();
    db.save(theme, None).expect("Couldn't save");
    let mut check : Theme = db.find_one_key_value("name", "autumn").unwrap().unwrap();
    assert_eq!(old_screen, check.screen);
    check.name = "fall".to_string();
    db.save(check, None).expect("Couldn't save");
}
