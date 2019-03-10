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

    add_material(instance, 80.);
    add_product(instance, 0, 10., 0, 1.0);
    assert_eq!(order_product(instance, 0, 8., 0, 0, true), 1);
}

#[test]
fn order_nenough_mat() {
    let instance = &mut resource_mng::init();

    add_material(instance, 79.);
    add_product(instance, 0, 10., 0, 1.0);
    assert_eq!(order_product(instance, 0, 8., 0, 0, true), 4);
}

#[test]
fn order_two_same_mat() {
    let instance = &mut resource_mng::init();

    add_material(instance, 80.);
    add_product(instance, 0, 10., 0, 1.0);
    add_product(instance, 0, 10., 0, 1.0);
    assert_eq!(order_product(instance, 0, 7., 0, 0, true), 1);
    //process_queue(instance);
    assert_eq!(order_product(instance, 1, 1., 0, 0, true), 1);
    process_queue(instance);
    let material = instance.get_material(0);
    assert_eq!(material.demand, 0.);
    assert_eq!(material.supply, 0.);
    assert_eq!(material.scarcity_cache, 50.0);
}