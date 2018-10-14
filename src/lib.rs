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

#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;

#[derive(Debug)]
#[repr(C)]
pub struct Product {
    //name: String,
    variants: ProductVariant, //change to Vec in the future
    //scarcity: usize,
    supply: usize,
    demand: usize,
    priority: usize,
}

impl Product {
    fn manufacture(&mut self, material: &mut Material, amount: &usize) {
        //let material = materials.get_mut(&self.variants.material_amount.0).unwrap();
        material.supply -= self.variants.material_and_amount.1 * amount;
        material.demand -= self.variants.material_and_amount.1 * amount;
        self.supply += amount;
        //product.demand -= amount;
    }

    fn deliver (&mut self, amount: &usize) {
        self.supply -= amount;
        self.demand -= amount;
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct ProductVariant {
    pub material_and_amount: (u64, usize), //change to materials in the future
    work_complexity: u8,
}

#[derive(Debug)]
#[repr(C)]
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

lazy_static! {
    static ref INSTANCE: Instance = Instance {
        materials: HashMap::new(),
        products: HashMap::new(),
        production_queue: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
        verbose: false,
    };
}

#[repr(C)]
pub struct Instance {
    materials: HashMap<u64,Material>,
    products: HashMap<u64,Product>,
    production_queue: [Vec<(u64, usize)>; 4],
    pub verbose: bool,
}

impl Instance {
    #[no_mangle]
    pub extern fn add_material(&mut self, name: u64, supply: usize) -> &'static u8 {
        //if name.trim().is_empty() { return &1; }
        if supply == 0 { return &2; }

        if !self.materials.contains_key(&name) {
            self.materials.insert(name.clone(),Material{
                //name,
                scarcity_cache: 0,
                demand: 0,
                supply,
                //deposit_size,
            });
            &0 //ok
        } else { &3 }

    }

    #[no_mangle]
    pub extern fn add_product(&mut self, name: u64, material_id: u64,
                              work_complexity: u8, material_amount: usize,
                              priority: usize) -> &'static u8 {
        //if name.trim().is_empty() { return &1 }
        //if material_id.trim().is_empty() { return &2 }
        if material_amount == 0 { return &3 }
        if !self.materials.contains_key(&material_id) { return &4 }
        if self.products.contains_key(&name) { return &5 }
        self.products.insert(name.clone(), Product{
            //name,
            variants: ProductVariant {
                material_and_amount: (material_id, material_amount),
                work_complexity,
            },
            supply: 0,
            demand: 0,
            priority,
        });
        &0
    }

    #[no_mangle]
    pub extern fn order_product(&mut self, name: u64, amount: usize) -> &'static u8 {
        if amount == 0 { return &2}
        let products = &mut self.products;
        if products.len() == 0 { panic!("no products in database");}
        let mut prod = products.remove(&name).unwrap();
        let production_queue = &mut self.production_queue;
        let mut material = match self.materials.remove(&prod.variants.material_and_amount.0) {
            Some(m) => m,
            None => return &3, //No such material in database.
        };
        prod.demand += amount;
        material.demand += amount * prod.variants.material_and_amount.1;
        material.scarcity_cache = material.calculate_scarcity();

        let mut code = &1;
        if amount <= prod.supply {
            prod.deliver(&amount);
            //code = &0; //ok
            panic!("cannot happen right now");
        } else {
            if material.supply < (amount * prod.variants.material_and_amount.1) //only a dev safeguard
                { //mat.demand -= amount * prod.types.material_amount.1;
                    code = &4; //Material not available.
                }
            if material.scarcity_cache > 50
                { //mat.demand -= amount * prod.types.material_amount.1;
                    code = &5; //Material scarce.
                }

            production_queue[prod.priority].push((name, amount));
            self.materials.insert(prod.variants.material_and_amount.0.clone(), material);
            products.insert(name, prod);
            for q in production_queue.iter_mut() {
                let mut i:usize = 0;
                //let mut to_remove = Vec::new();
                while i != q.len() {
                    let mut q_product = products.get_mut(&q[i].0).unwrap();
                    let mut q_material = self.materials.get_mut(&q_product.variants.material_and_amount.0).unwrap();
                    if q_material.supply >= q[i].1 * q_product.variants.material_and_amount.1 &&
                        q_material.scarcity_cache <= 50 {
                        q_product.manufacture(q_material, &q[i].1);
                        q_product.deliver(&q[i].1);
                        //i.1 = 0;
                        //to_remove.push(cnt);
                        let tmp = q.remove(i);
                        if self.verbose { println!(" * Manufacturing {}x product \"{}\" from priority {} production queue.", tmp.1, tmp.0, q_product.priority+1);}
                    }
                    else { i += 1; }
                }
            }
        }
        return code;
    }

    //pub fn is_in_supply() {}

    #[no_mangle]
    pub extern fn update_supply(&mut self, name: &u64, amount: usize) -> bool {
        match self.materials.get_mut(name) {
            Some(x) => {
                x.supply = amount;
                true
            },
            None => false
        }
    }

    //pub fn update_material_deposit_size() {}

    #[no_mangle]
    pub extern fn get_material_count (&self) -> usize {
        self.materials.len()
    }

    #[no_mangle]
    pub extern fn get_material_demand (&self, name: &u64) -> usize {
        self.materials.get(name).unwrap().demand
    }

    #[no_mangle]
    pub extern fn get_material_supply (&self, name: &u64) -> usize {
        self.materials.get(name).unwrap().supply
    }

    #[no_mangle]
    pub extern fn get_material_scarcity (&mut self, name: &u64) -> usize {
        self.materials.get_mut(name).unwrap().scarcity_cache
    }

    #[no_mangle]
    pub extern fn get_product_count (&self) -> usize {
        self.products.len()
    }

    #[no_mangle]
    pub extern fn get_product_types (&self, name: &u64) -> &ProductVariant {
        &self.products.get(name).unwrap().variants
    }

    #[no_mangle]
    pub extern fn tst_set_product_supply(&mut self, name: &u64, count: usize) {
        self.products.get_mut(name).unwrap().supply = count;
    }

    #[no_mangle]
    pub extern fn tst_get_material(&self, name: &u64) -> &Material {
        self.materials.get(name).unwrap()
    }

    #[no_mangle]
    pub extern fn tst_get_materials(&self) -> &HashMap<u64, Material> {
        &self.materials
    }

    #[no_mangle]
    pub extern fn tst_get_products(&self) -> &HashMap<u64, Product> {
        &self.products
    }

}

#[no_mangle]
pub extern fn init() -> Instance {
    Instance {
        materials: HashMap::new(),
        products: HashMap::new(),
        production_queue: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
        verbose: false,
    }
}

#[no_mangle]
pub extern fn load(materials: HashMap<u64, Material>,
                   products: HashMap<u64, Product>,
                   production_queue: [Vec<(u64, usize)>; 4],
                   verbose: bool) -> Instance {
    Instance {
        materials,
        products,
        production_queue,
        verbose,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_same_material() {
        let mut instance = init();
        instance.add_material(1234, 8);
        instance.add_material(1234, 1);
        assert_eq!(instance.materials.get(&1234).unwrap().supply, 8);
    }

    #[test]
    fn add_product_without_material() {
        let mut instance = init();
        assert_ne!(!instance.add_product(1234, 12345, 5, 10, 0),0);
    }

    #[test]
    fn add_same_product() {
        let mut instance = init();
        instance.add_material(1234, 8);
        instance.add_product(12345, 1234, 5, 10, 0);
        instance.add_product(12345, 1234, 0, 5, 0);
        assert_eq!(instance.products.get(&12345).unwrap().variants.material_and_amount.1, 10);
    }

    #[test]
    fn add_prod_zero_mat() {
        let mut instance = init();
        instance.add_material(1234, 8);
        assert_ne!(instance.add_product(1234, 1234, 5, 0, 0), &0);
    }
}