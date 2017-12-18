extern crate resource_mng;

#[test]
fn demand_enough_prod() {
    let mut instance = resource_mng::init();

    instance.add_material(String::from("ore"),81);
    instance.add_product(String::from("steel"), String::from("ore"), 5, 10);
    instance.tst_set_product_supply("steel", 8);
    assert!(instance.demand_product("steel", 8).unwrap());
}

#[test]
fn demand_enough_mat() {
    let mut instance = resource_mng::init();

    instance.add_material(String::from("ore"),80);
    instance.add_product(String::from("steel"), String::from("ore"), 5, 10);
    assert!(!instance.demand_product("steel", 8).unwrap());
}

#[test]
fn demand_nenough_mat() {
    let mut instance = resource_mng::init();

    instance.add_material(String::from("ore"),79);
    instance.add_product(String::from("steel"), String::from("ore"), 5, 10);
    assert_eq!(instance.demand_product("steel", 8), Err("Materials scarce."));
}