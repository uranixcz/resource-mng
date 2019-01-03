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

#[derive(Debug)]
pub struct Product {
    //name: String,
    variants: Vec<ProductVariant>, //change to Vec in the future
    //scarcity: usize,
    supply: usize,
    demand: usize,
    priority: usize,
}

impl Product {
    fn manufacture(&mut self, material: &mut Material, amount: &usize) {
        //let material = materials.get_mut(&self.variants.material_amount.0).unwrap();
        material.supply -= self.variants.first().unwrap().material_and_amount.1 * amount;
        material.demand -= self.variants.first().unwrap().material_and_amount.1 * amount;
        self.supply += amount;
        //product.demand -= amount;
    }

    fn deliver (&mut self, amount: &usize) {
        self.supply -= amount;
        self.demand -= amount;
    }
}

#[derive(Debug)]
#[derive(Copy, Clone)]
pub struct ProductVariant {
    pub material_and_amount: (usize, usize), //change to materials in the future
    //work_complexity: u8,
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
    pub extern fn calculate_scarcity(&self) -> usize { //100/2=50
        if self.supply != 0 {
            self.demand * 50 / (self.supply /*+ self.deposit_size*/)
        } else { usize::max_value() }
    }
}

pub struct Instance {
    materials: HashMap<usize,Material>,
    products: HashMap<usize,Product>,
    production_queue: [Vec<(usize, usize)>; 4],
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
            material_and_amount: (material_id, material_amount),
            //work_complexity,
        }],
        supply: 0,
        demand: 0,
        priority,
    });
    0
}

#[no_mangle]
pub extern fn order_product(instance: &mut Instance, id: usize, amount: usize) -> u8 {
    if amount == 0 { return 2}
    let products = &mut instance.products;
    if products.len() == 0 { panic!("no products in database");}
    let mut prod = products.remove(&id).unwrap();
    let production_queue = &mut instance.production_queue;
    let mut material = match instance.materials.remove(&prod.variants.first().unwrap().material_and_amount.0) {
        Some(m) => m,
        None => return 3, //No such material in database.
    };
    prod.demand += amount;
    material.demand += amount * prod.variants.first().unwrap().material_and_amount.1;
    material.scarcity_cache = material.calculate_scarcity();

    let mut code = 1;
    if amount <= prod.supply {
        prod.deliver(&amount);
        //code = &0; //ok
        panic!("cannot happen right now");
    } else {
        if material.supply < (amount * prod.variants.first().unwrap().material_and_amount.1) //only a dev safeguard
            { //mat.demand -= amount * prod.types.material_amount.1;
                code = 4; //Material not available.
            }
        if material.scarcity_cache > 50
            { //mat.demand -= amount * prod.types.material_amount.1;
                code = 5; //Material scarce.
            }

        production_queue[prod.priority].push((id, amount));
        instance.materials.insert(prod.variants.first().unwrap().material_and_amount.0.clone(), material);
        products.insert(id, prod);
        for q in production_queue.iter_mut() {
            let mut i:usize = 0;
            //let mut to_remove = Vec::new();
            while i != q.len() {
                let mut q_product = products.get_mut(&q[i].0).unwrap();

                for variant in q_product.variants.clone() {
                    let mut q_material = instance.materials.get_mut(&variant.material_and_amount.0).unwrap();
                    if q_material.supply >= q[i].1 * variant.material_and_amount.1 &&
                        q_material.scarcity_cache <= 50 {
                        q_product.manufacture(q_material, &q[i].1);
                        q_product.deliver(&q[i].1);
                        //i.1 = 0;
                        //to_remove.push(cnt);
                        let tmp = q.remove(i);
                        if instance.verbose {
                            println!(" * Manufacturing {}x product #{} from priority {} production queue.",
                                     tmp.1, tmp.0, q_product.priority+1);
                        }
                    }
                    else { i += 1; }
                }
            }
        }
    }
    return code;
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
pub extern fn add_product_variant(instance: &mut Instance, id: usize, material_id: usize, material_amount: usize) -> u8 {
    //instance.products.get_mut(&id).unwrap()
    if !instance.products.contains_key(&id) { return 1 }
    if !instance.materials.contains_key(&material_id) { return 2 }
    instance.products.get_mut(&id).unwrap().variants.push(ProductVariant {
        material_and_amount: (material_id, material_amount)
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
pub extern fn get_product_variant (instance: &Instance, id: &usize) -> [usize; 2] {
    let tmp = instance.products.get(id).unwrap().variants.first().unwrap().material_and_amount;
    [tmp.0, tmp.1]
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
        assert_eq!(instance.products.get(&12345).unwrap().variants.first().unwrap().material_and_amount.1, 10);
    }

    #[test]
    fn add_prod_zero_mat() {
        let instance = &mut init();
        add_material(instance, 1234, 8);
        assert_ne!(add_product(instance, 1234, 1234, 0, 0), 0);
    }
}