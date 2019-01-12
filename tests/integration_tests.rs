extern crate resource_mng;

use resource_mng::*;

/*#[test]
fn order_enough_prod() {
    let mut instance = resource_mng::init();

    instance.add_material(123,80);
    instance.add_product(1234, 123, 5, 10, 0);
    instance.tst_set_product_supply(&1234, 8);
    assert_eq!(instance.order_product(1234, 8), &0);
}*/

#[test]
fn order_enough_mat() {
    let instance = &mut resource_mng::init();

    add_material(instance, 123,80);
    add_product(instance, 1234, 123, 10, 0);
    assert_eq!(order_product(instance, 1234, 8, 0), 1);
}

#[test]
fn order_nenough_mat() {
    let instance = &mut resource_mng::init();

    add_material(instance, 123, 79);
    add_product(instance, 1234, 123, 10, 0);
    assert_eq!(order_product(instance, 1234, 8, 0), 4);
}

#[test]
fn order_two_same_mat() {
    let instance = &mut resource_mng::init();

    add_material(instance, 123,80);
    add_product(instance, 1234, 123, 10, 0);
    add_product(instance, 1235, 123, 10, 0);
    assert_eq!(order_product(instance, 1234, 7, 0), 1);
    //process_queue(instance);
    assert_eq!(order_product(instance, 1235, 1, 0), 1);
    //process_queue(instance);
    assert_eq!(tst_get_material(instance, &123).demand, 0);
    assert_eq!(tst_get_material(instance, &123).supply, 0);
    assert_eq!(tst_get_material(instance, &123).scarcity_cache, 50);
}