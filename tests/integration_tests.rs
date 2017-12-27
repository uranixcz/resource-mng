extern crate resource_mng;

#[test]
fn order_enough_prod() {
    let mut instance = resource_mng::init();

    instance.add_material(String::from("ore"),80);
    instance.add_product(String::from("steel"), String::from("ore"), 5, 10);
    instance.tst_set_product_supply("steel", 8);
    assert!(instance.order_product("steel", 8).unwrap());
}

#[test]
fn order_enough_mat() {
    let mut instance = resource_mng::init();

    instance.add_material(String::from("ore"),80);
    instance.add_product(String::from("steel"), String::from("ore"), 5, 10);
    assert!(!instance.order_product("steel", 8).unwrap());
}

#[test]
fn order_nenough_mat() {
    let mut instance = resource_mng::init();

    instance.add_material(String::from("ore"),79);
    instance.add_product(String::from("steel"), String::from("ore"), 5, 10);
    assert_eq!(instance.order_product("steel", 8), Err("Material scarce."));
}

#[test]
fn order_two_same_mat() {
    let mut instance = resource_mng::init();

    instance.add_material(String::from("wood"),80);
    instance.add_product(String::from("chair"), String::from("wood"), 5, 10);
    instance.add_product(String::from("table"), String::from("wood"), 5, 10);
    assert!(!instance.order_product("chair", 7).unwrap());
    assert!(!instance.order_product("table", 1).unwrap());
    assert_eq!(instance.tst_get_material_params("wood").demand, 0);
    assert_eq!(instance.tst_get_material_params("wood").supply, 0);
    assert_eq!(instance.tst_get_material_params("wood").scarcity, 50);
}