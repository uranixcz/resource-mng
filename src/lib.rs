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

use std::collections::HashMap;
use std::cmp::Ordering;

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

    fn deliver (&mut self, amount: usize) {
        self.supply -= amount;
        self.demand -= amount;
    }

    pub fn get_variant (&self, variant_id: usize) -> &ProductVariant {
        self.variants.iter().find(|x| x.id == variant_id).unwrap()
    }
}

//#[derive(Debug)]
#[derive(Copy, Clone, Eq)]
pub struct ProductVariant {
    id: usize,
    pub components: Component, //change to vec in the future
    //work_complexity: u8,
}

impl Ord for ProductVariant {
    fn cmp(&self, other: &ProductVariant) -> Ordering {
        self.components.scarcity_cache.cmp(&other.components.scarcity_cache)
    }
}

impl PartialOrd for ProductVariant {
    fn partial_cmp(&self, other: &ProductVariant) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ProductVariant {
    fn eq(&self, other: &ProductVariant) -> bool {
        self.components.scarcity_cache == other.components.scarcity_cache
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Component {
    pub material_id: usize,
    pub material_amount: usize,
    pub scarcity_cache: usize,
}

#[derive(Debug)]
#[derive(Copy, Clone)]
pub struct Material {
    //name: String,
    pub scarcity_cache: usize,
    pub demand: usize,
    pub supply: usize,
    //deposit_size: usize,
}

impl Material {
    #[must_use]
    pub extern fn get_scarcity(&self) -> usize { //100/2=50
        if self.supply != 0 {
            self.demand * 50 / (self.supply /*+ self.deposit_size*/)
        } else { usize::max_value() }
    }
}

struct Order {
    product_id: usize,
    product_amount: usize,
    preferred_variant: usize,
}

pub struct Instance {
    materials: HashMap<usize,Material>,
    products: HashMap<usize,Product>,
    production_queue: [Vec<Order>; 4],
    pub verbose: bool,
}

#[no_mangle]
pub extern fn init() -> Box<Instance> {
    Box::from(Instance {
        materials: HashMap::new(),
        products: HashMap::new(),
        production_queue: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
        verbose: false,
    })
}

#[no_mangle]
pub extern fn add_material(instance: &mut Instance, new_id: usize, supply: usize) -> u8 {
    //if name.trim().is_empty() { return &1; }
    if supply == 0 { return 2; }

    if !instance.materials.contains_key(&new_id) {
        instance.materials.insert(new_id,Material{
            //name,
            scarcity_cache: 0,
            demand: 0,
            supply,
            //deposit_size,
        });
        0 //ok
    } else { 3 }

}

#[no_mangle]
pub extern fn add_product(instance: &mut Instance, new_id: usize, material_id: usize, material_amount: usize, priority: usize) -> u8 {
    //if name.trim().is_empty() { return &1 }
    //if material_id.trim().is_empty() { return &2 }
    if material_amount == 0 { return 3 }
    if !instance.materials.contains_key(&material_id) { return 4 }
    if instance.products.contains_key(&new_id) { return 5 }
    instance.products.insert(new_id, Product{
        //name,
        variants: vec![ProductVariant {
            id: 0,
            components: Component { material_id, material_amount, scarcity_cache: 0 },
            //work_complexity,
        }],
        supply: 0,
        demand: 0,
        priority,
    });
    0
}

#[no_mangle]
pub extern fn order_product(instance: &mut Instance, id: usize, amount: usize, variant_id: usize) -> u8 {
    if amount == 0 { return 2}
    let products = &mut instance.products;
    if products.len() == 0 { panic!("no products in database");}
    let mut prod = products.remove(&id).unwrap();

    let variant = prod.get_variant(variant_id).clone();
    let production_queue = &mut instance.production_queue;
    let mut material = match instance.materials.remove(&variant.components.material_id) {
        Some(m) => m,
        None => return 3, //No such material in database.
    };
    prod.demand += amount;
    material.demand += amount * variant.components.material_amount;
    material.scarcity_cache = material.get_scarcity();

    let mut code = 1;
    if amount <= prod.supply {
        prod.deliver(amount);
        //code = &0; //ok
        panic!("cannot happen right now");
    } else {
        if material.supply < (amount * variant.components.material_amount) //only a dev safeguard
        { //mat.demand -= amount * prod.types.material_amount.1;
            code = 4; //Material not available.
        }
        if material.scarcity_cache > 50
        { //mat.demand -= amount * prod.types.material_amount.1;
            code = 5; //Material scarce.
        }

        production_queue[prod.priority].push(Order { product_id: id, product_amount: amount, preferred_variant: variant_id });
    }
    instance.materials.insert(variant.components.material_id.clone(), material);
    products.insert(id, prod);
    return code;
}

#[no_mangle]
pub extern fn process_queue(instance: &mut Instance) {
    for q in instance.production_queue.iter_mut() {
        let mut i:usize = 0;
        //let mut to_remove = Vec::new();
        while i != q.len() {
            let mut found = false;
            let q_product = instance.products.get_mut(&q[i].product_id).unwrap();

            for variant in q_product.variants.iter_mut() {
                let variant_material = instance.materials.get_mut(&variant.components.material_id).unwrap();
                variant_material.scarcity_cache = variant_material.get_scarcity();
                variant.components.scarcity_cache = variant_material.scarcity_cache;
            }
            q_product.variants.sort_unstable();
            let swap = q_product.variants.iter().position(|x| x.id == q[i].preferred_variant).unwrap();
            q_product.variants.swap(0, swap);

            for variant in q_product.variants.clone() {
                let q_material = instance.materials.get_mut(&variant.components.material_id).unwrap();
                let material_amount = q[i].product_amount * variant.components.material_amount;
                if q_material.supply >= material_amount {
                    if variant.id != q[i].preferred_variant {q_material.demand += material_amount;}
                    q_product.manufacture(q_material, q[i].product_amount, &variant);
                    q_product.deliver(q[i].product_amount);
                    let tmp = q.remove(i);
                    if instance.verbose {
                        println!(" * Manufacturing {}x product #{}, variant {} from priority {} production queue.",
                                 tmp.product_amount, tmp.product_id, variant.id, q_product.priority+1);
                    }
                    found = true;
                    break;
                }
            }
            if !found { i += 1; }
        }
    }
}

//pub fn is_in_supply() {}

#[no_mangle]
pub extern fn update_supply(instance: &mut Instance, id: usize, amount: usize) -> bool {
    match instance.materials.get_mut(&id) {
        Some(x) => {
            x.supply = amount;
            true
        },
        None => false
    }
}

#[no_mangle]
pub extern fn add_product_variant(instance: &mut Instance, product_id: usize, material_id: usize, material_amount: usize) -> u8 {
    //instance.products.get_mut(&id).unwrap()
    if !instance.products.contains_key(&product_id) { return 1 }
    if !instance.materials.contains_key(&material_id) { return 2 }
    let product = instance.products.get_mut(&product_id).unwrap();
    let variant_id = product.variants.len(); //this must be changed when remove_product_variant is implemented!
    product.variants.push(ProductVariant {
        id: variant_id,
        components: Component {material_id, material_amount, scarcity_cache: 0}
    });
    0
}

//pub fn update_material_deposit_size() {}

#[no_mangle]
pub extern fn get_material_count (instance: &Instance) -> usize {
    instance.materials.len()
}

#[no_mangle]
pub extern fn get_material_demand (instance: &Instance, id: &usize) -> usize {
    instance.materials.get(id).unwrap().demand
}

#[no_mangle]
pub extern fn get_material_supply (instance: &Instance, id: &usize) -> usize {
    instance.materials.get(id).unwrap().supply
}

#[no_mangle]
pub extern fn get_material_scarcity (instance: &mut Instance, id: &usize) -> usize {
    instance.materials.get_mut(id).unwrap().scarcity_cache
}

#[no_mangle]
pub extern fn get_product_count (instance: &Instance) -> usize {
    instance.products.len()
}

#[no_mangle]
pub extern fn get_product_supply (instance: &Instance, id: &usize) -> usize {
    instance.products.get(id).unwrap().supply
}

#[no_mangle]
pub extern fn get_product_demand (instance: &Instance, id: &usize) -> usize {
    instance.products.get(id).unwrap().demand
}

#[no_mangle]
pub extern fn get_product_priority (instance: &Instance, id: &usize) -> usize {
    instance.products.get(id).unwrap().priority
}

#[no_mangle]
pub extern fn get_product_variant (instance: &Instance, product_id: &usize, variant_id: usize) -> [usize; 2] {
    let tmp: Component = instance.products.get(product_id).unwrap().variants.get(variant_id).unwrap().components; //fixme
    [tmp.material_id, tmp.material_amount]
}

pub fn get_product_variants<'a>(instance: &'a Instance, id: &usize) -> &'a Vec<ProductVariant> {
    &instance.products.get(id).unwrap().variants
}

#[no_mangle]
pub extern fn tst_set_product_supply(instance: &mut Instance, id: &usize, count: usize) {
    instance.products.get_mut(id).unwrap().supply = count;
}

pub fn tst_get_material(instance: &Instance, id: &usize) -> Material {
    instance.materials.get(id).unwrap().clone()
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
        assert_ne!(!add_product(instance, 1234, 12345, 10, 0),0);
    }

    #[test]
    fn add_same_product() {
        let instance = &mut init();
        add_material(instance, 1234, 8);
        add_product(instance, 12345, 1234, 10, 0);
        add_product(instance, 12345, 1234, 5, 0);
        assert_eq!(instance.products.get(&12345).unwrap().variants.first().unwrap().components.material_amount, 10);
    }

    #[test]
    fn add_prod_zero_mat() {
        let instance = &mut init();
        add_material(instance, 1234, 8);
        assert_ne!(add_product(instance, 1234, 1234, 0, 0), 0);
    }
}