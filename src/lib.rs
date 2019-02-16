/*
* Copyright 2017-2018 Michal Mauser
*
* This program is free software: you can redistribute it and/or modify
* it under the terms of the GNU Affero General Public License as published by
* the Free Software Foundation, either version 3 of the License, or
* (at your option) any later version.
*
* This program is distributed in the hope that it will be useful,
* but WITHOUT ANY WARRANTY; without even the implied warranty of
* MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
* GNU Affero General Public License for more details.
*
* You should have received a copy of the GNU Affero General Public License
* along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

mod internals;

use std::collections::HashMap;
use std::cmp::Ordering;
use std::collections::hash_map::Entry;

const PRIORITIES: usize = 4;
const EQUILIBRIUM: usize = 50;

//#[derive(Debug)]
pub struct Product {
    //name: String,
    variants: Vec<ProductVariant>,
    //scarcity: usize,
    supply: usize,
    demand: usize,
    priority: usize,
}

impl Product {
    fn manufacture(&mut self, material: &mut Material, amount: usize, variant: &ProductVariant) {
        let material_amount = variant.components.material_amount * amount;
        material.supply -= material_amount;
        material.demand -= material_amount;
        self.supply += amount;
        //product.demand -= amount;
    }

    /*fn manufacture_by_id(&mut self, material: &mut Material, amount: usize, variant_id: usize) {
        let index = self.variants.iter().position(|x| x.id == variant_id).unwrap();
        let material_amount = self.variants[index].material_and_amount.1 * amount;
        material.supply -= material_amount;
        material.demand -= material_amount;
        self.supply += amount;
        //product.demand -= amount;
    }*/

    fn deliver(&mut self, amount: usize) {
        self.supply -= amount;
        self.demand -= amount;
    }

    pub fn get_variant(&self, variant_id: usize) -> &ProductVariant {
        self.variants.iter().find(|x| x.id == variant_id).unwrap()
    }
}

//#[derive(Debug)]
#[derive(Copy, Clone)]
pub struct ProductVariant {
    id: usize,
    pub components: Component,
    //change to vec in the future
    work_complexity: f64,
}

impl PartialOrd for ProductVariant {
    fn partial_cmp(&self, other: &ProductVariant) -> Option<Ordering> {
        let my = self.components.scarcity_cache as f64 / self.work_complexity;
        let other = other.components.scarcity_cache as f64 / other.work_complexity;
        my.partial_cmp(&other)
    }
}

impl PartialEq for ProductVariant {
    fn eq(&self, other: &ProductVariant) -> bool {
        let my = self.components.scarcity_cache as f64 / self.work_complexity;
        let other = other.components.scarcity_cache as f64 / other.work_complexity;
        my == other
    }
}

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Component {
    pub material_id: usize,
    pub material_amount: usize,
    pub scarcity_cache: usize,
}

#[derive(Debug)]
#[derive(Copy, Clone)]
pub struct Material {
    pub scarcity_cache: usize,
    pub demand: usize,
    pub supply: usize,
    //deposit_size: usize,
}

impl Material {
    #[must_use]
    pub extern fn get_scarcity(&self) -> usize { //100/2=50
        if self.supply != 0 {
            self.demand * EQUILIBRIUM / (self.supply /*+ self.deposit_size*/)
        } else { usize::max_value() }
    }
}

#[repr(C)]
pub struct Order {
    product_id: usize,
    product_amount: usize,
    preferred_variant: usize,
    user_id: usize,
    allow_substitution: bool,
}

#[repr(C)]
pub struct COption<T> {
    is_some: bool,
    data: T,
}

pub struct Instance {
    materials: HashMap<usize, Material>,
    products: HashMap<usize, Product>,
    production_queue: [Vec<Order>; PRIORITIES],
    finished_products: Vec<Order>,
    pub verbose: usize,
}

#[no_mangle]
pub extern fn init() -> Box<Instance> {
    Box::from(Instance {
        materials: HashMap::new(),
        products: HashMap::new(),
        production_queue: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
        finished_products: Vec::new(),
        verbose: 0,
    })
}

#[no_mangle]
pub extern fn add_material(instance: &mut Instance, new_id: usize, supply: usize) -> u8 {
    const ZERO_SUPPLY: u8 = 2;
    const DUPLICATE_MATERIAL: u8 = 3;

    if supply == 0 { return ZERO_SUPPLY; }

    match instance.materials.entry(new_id) {
        Entry::Vacant(v) => {
            v.insert(Material {
                scarcity_cache: 0,
                demand: 0,
                supply,
            });
            0 //ok
        }
        Entry::Occupied(_) => DUPLICATE_MATERIAL
    }
}

#[no_mangle]
pub extern fn add_product(instance: &mut Instance, new_id: usize, material_id: usize, material_amount: usize, priority: usize, work_complexity: f64) -> u8 {
    const ZERO_MATERIAL: u8 = 3;
    const NO_SUCH_MATERIAL: u8 = 4;
    const NO_SUCH_PRODUCT: u8 = 5;

    if material_amount == 0 { return ZERO_MATERIAL; }
    if !instance.materials.contains_key(&material_id) { return NO_SUCH_MATERIAL; }
    if instance.products.contains_key(&new_id) { return NO_SUCH_PRODUCT; }
    instance.products.insert(new_id, Product {
        //name,
        variants: vec![ProductVariant {
            id: 0,
            components: Component { material_id, material_amount, scarcity_cache: 0 },
            work_complexity,
        }],
        supply: 0,
        demand: 0,
        priority,
    });
    0
}

#[no_mangle]
pub extern fn order_product(instance: &mut Instance,
                            id: usize,
                            amount: usize,
                            variant_id: usize,
                            user_id: usize,
                            allow_substitution: bool) -> u8
{
    const OK_QUEUE: u8 = 1;
    const CANNOT_ORDER_0_PRODUCTS: u8 = 2;
    const NO_SUCH_MATERIAL: u8 = 3;
    const MATERIAL_NOT_AVAIL: u8 = 4;
    const MATERIAL_SCARCE: u8 = 5;

    if amount == 0 { return CANNOT_ORDER_0_PRODUCTS; }
    let products = &mut instance.products;
    if products.is_empty() { panic!("no products in database"); }
    let mut prod = products.remove(&id).unwrap();

    let variant = *prod.get_variant(variant_id);
    let production_queue = &mut instance.production_queue;
    let mut material = match instance.materials.remove(&variant.components.material_id) {
        Some(m) => m,
        None => return NO_SUCH_MATERIAL, //No such material in database.
    };
    prod.demand += amount;
    material.demand += amount * variant.components.material_amount;
    material.scarcity_cache = material.get_scarcity();

    let mut code = OK_QUEUE;
    if amount <= prod.supply {
        prod.deliver(amount);
        //code = 0; //ok, already manufactured
        panic!("cannot happen right now");
    } else {
        if material.supply < (amount * variant.components.material_amount) //only a dev safeguard
        {
            code = MATERIAL_NOT_AVAIL; //Material not available.
        }
        if material.scarcity_cache > EQUILIBRIUM
        {
            code = MATERIAL_SCARCE; //Material scarce.
        }

        production_queue[prod.priority].push(Order {
            product_id: id,
            product_amount: amount,
            preferred_variant: variant_id,
            user_id,
            allow_substitution,
        });
    }
    instance.materials.insert(variant.components.material_id, material);
    products.insert(id, prod);

    if code != OK_QUEUE {
        internals::process_queue(&mut instance.production_queue,
                                 &mut instance.products,
                                 &mut instance.materials,
                                 &mut instance.finished_products,
                                 instance.verbose);
    }

    code
}

#[no_mangle]
pub extern fn process_queue(instance: &mut Instance) {
    internals::process_queue(&mut instance.production_queue,
                             &mut instance.products,
                             &mut instance.materials,
                             &mut instance.finished_products,
                             instance.verbose);
}

//pub fn is_in_supply() {}

#[no_mangle]
pub extern fn update_supply(instance: &mut Instance, id: usize, amount: usize) -> bool {
    let result = match instance.materials.get_mut(&id) {
        Some(x) => {
            x.supply = amount;
            true
        }
        None => false
    };
    internals::process_queue(&mut instance.production_queue,
                             &mut instance.products,
                             &mut instance.materials,
                             &mut instance.finished_products,
                             instance.verbose);

    result
}

#[no_mangle]
pub extern fn add_product_variant(instance: &mut Instance, product_id: usize, material_id: usize, material_amount: usize, work_complexity: f64) -> u8 {
    const NO_SUCH_PRODUCT: u8 = 1;
    const NO_SUCH_MATERIAL: u8 = 2;

    if !instance.products.contains_key(&product_id) { return NO_SUCH_PRODUCT; }
    if !instance.materials.contains_key(&material_id) { return NO_SUCH_MATERIAL; }
    let product = instance.products.get_mut(&product_id).unwrap();
    let variant_id = product.variants.len(); //this must be changed when remove_product_variant is implemented!
    product.variants.push(ProductVariant {
        id: variant_id,
        components: Component { material_id, material_amount, scarcity_cache: 0 },
        work_complexity,
    });
    0
}

//pub fn update_material_deposit_size() {}

#[no_mangle]
pub extern fn get_material_count(instance: &Instance) -> usize {
    instance.materials.len()
}

#[no_mangle]
pub extern fn get_material_demand(instance: &Instance, id: &usize) -> usize {
    instance.materials[id].demand
}

#[no_mangle]
pub extern fn get_material_supply(instance: &Instance, id: &usize) -> usize {
    instance.materials[id].supply
}

#[no_mangle]
pub extern fn get_material_scarcity(instance: &mut Instance, id: &usize) -> usize {
    instance.materials.get_mut(id).unwrap().scarcity_cache
}

#[no_mangle]
pub extern fn get_product_count(instance: &Instance) -> usize {
    instance.products.len()
}

#[no_mangle]
pub extern fn get_product_supply(instance: &Instance, id: &usize) -> usize {
    instance.products[id].supply
}

#[no_mangle]
pub extern fn get_product_demand(instance: &Instance, id: &usize) -> usize {
    instance.products[id].demand
}

#[no_mangle]
pub extern fn get_product_priority(instance: &Instance, id: &usize) -> usize {
    instance.products[id].priority
}

#[no_mangle]
pub extern fn get_product_variant(instance: &Instance, product_id: &usize, variant_id: usize) -> Component {
    instance.products[product_id].variants[variant_id].components
}

#[no_mangle]
pub extern fn get_next_finished(instance: &mut Instance) -> COption<Order> {
    match instance.finished_products.pop() {
        Some(p) => COption { is_some: true, data: p },
        None => COption {
            is_some: false,
            data: Order {
                product_id: 0,
                product_amount: 0,
                preferred_variant: 0,
                user_id: 0,
                allow_substitution: false,
            },
        }
    }
}

pub fn get_product_variants<'a>(instance: &'a Instance, id: &usize) -> &'a Vec<ProductVariant> {
    &instance.products[id].variants
}

#[no_mangle]
pub extern fn tst_set_product_supply(instance: &mut Instance, id: &usize, count: usize) {
    instance.products.get_mut(id).unwrap().supply = count;
}

pub fn tst_get_material(instance: &Instance, id: usize) -> Material {
    instance.materials[&id].clone()
}

pub fn tst_get_materials(instance: &Instance) -> &HashMap<usize, Material> {
    &instance.materials
}

pub fn tst_get_products(instance: &Instance) -> &HashMap<usize, Product> {
    &instance.products
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_same_material() {
        let instance = &mut init();
        add_material(instance, 1234, 8);
        add_material(instance, 1234, 1);
        assert_eq!(instance.materials.get(&1234).unwrap().supply, 8);
    }

    #[test]
    fn add_product_without_material() {
        let instance = &mut init();
        assert_ne!(!add_product(instance, 1234, 12345, 10, 0, 1.0), 0);
    }

    #[test]
    fn add_same_product() {
        let instance = &mut init();
        add_material(instance, 1234, 8);
        add_product(instance, 12345, 1234, 10, 0, 1.0);
        add_product(instance, 12345, 1234, 5, 0, 1.0);
        assert_eq!(instance.products.get(&12345).unwrap().variants.first().unwrap().components.material_amount, 10);
    }

    #[test]
    fn add_prod_zero_mat() {
        let instance = &mut init();
        add_material(instance, 1234, 8);
        assert_ne!(add_product(instance, 1234, 1234, 0, 0, 1.0), 0);
    }
}