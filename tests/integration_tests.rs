extern crate resource_mng;

#[test]
/*fn order_enough_prod() {
    let mut instance = resource_mng::init();

    instance.add_material(123,80);
    instance.add_product(1234, 123, 5, 10, 0);
    instance.tst_set_product_supply(&1234, 8);
    assert_eq!(instance.order_product(1234, 8), &0);
}*/

#[test]
fn order_enough_mat() {
    let mut instance = resource_mng::init();

    instance.add_material(123,80);
    instance.add_product(1234, 123, 5, 10, 0);
    assert_eq!(instance.order_product(1234, 8), &1);
}

#[test]
fn order_nenough_mat() {
    let mut instance = resource_mng::init();

    instance.add_material(123,79);
    instance.add_product(1234, 123, 5, 10, 0);
    assert_eq!(instance.order_product(1234, 8), &4);
}

#[test]
fn order_two_same_mat() {
    let mut instance = resource_mng::init();

    instance.add_material(123,80);
    instance.add_product(1234, 123, 5, 10, 0);
    instance.add_product(1235, 123, 5, 10, 0);
    assert_eq!(instance.order_product(1234, 7), &1);
    assert_eq!(instance.order_product(1235, 1), &1);
    assert_eq!(instance.tst_get_material(&123).demand, 0);
    assert_eq!(instance.tst_get_material(&123).supply, 0);
    assert_eq!(instance.tst_get_material(&123).scarcity_cache, 50);
}