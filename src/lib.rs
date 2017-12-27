/*
* Copyright 2017 Michal Mauser
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
#[repr(C)]
pub struct Product {
    //name: String,
    types: ProductType, //change to Vec in the future
    //scarcity: usize,
    supply: usize,
    demand: usize,
}

#[derive(Debug)]
#[repr(C)]
pub struct ProductType {
    pub material_amount: (String, usize), //change to materials in the future
    work_complexity: u8,
}

#[derive(Debug)]
#[repr(C)]
pub struct Material {
    //name: String,
    pub scarcity: usize,
    pub demand: usize,
    pub supply: usize,
    //deposit_size: usize,
}

impl Material {
    fn calculate_scarcity(&mut self) { //100/2=50
        self.scarcity = if self.supply != 0 {
            self.demand * 50 / (self.supply /*+ self.deposit_size*/)
        } else { usize::max_value() }
    }
}

#[repr(C)]
pub struct Instance {
    materials: HashMap<String,Material>,
    products: HashMap<String,Product>,
}

impl Instance {
    #[no_mangle]
    pub extern fn add_material(&mut self, name: String, supply: usize) -> u8 {
        if name.trim().is_empty() { return 1; }
        if supply == 0 { return 2; }

        if !self.materials.contains_key(&name) {
            self.materials.insert(name.clone(),Material{
                //name,
                scarcity: 0,
                demand: 0,
                supply,
                //deposit_size,
            });
            0 //ok
        } else { 3 }

    }

    #[no_mangle]
    pub extern fn add_product(&mut self, name: String, material_id: String,
                              work_complexity: u8, material_amount: usize) -> bool {
        if name.trim().is_empty()
            || material_id.trim().is_empty()
            || material_amount == 0
            || !self.materials.contains_key(&material_id)
            || self.products.contains_key(&name) {
                false
            }
        else {
            self.products.insert(name.clone(), Product{
                //name,
                types: ProductType {
                    material_amount: (material_id, material_amount),
                    work_complexity,
                },
                supply: 0,
                demand: 0,
            });
            //println!("{:?}", self.products.get(&name).unwrap());
            true
        }
    }

    #[no_mangle]
    pub extern fn order_product(&mut self, name: &str, amount: usize) -> Result<bool, &'static str> {
        if amount == 0 { return Err("Cannot order 0 products.")}
        let prod = self.products.get_mut(name).unwrap();
        let mat = match self.materials.get_mut(&prod.types.material_amount.0) {
            Some(m) => m,
            None => return Err("No such material in database."),
        };
        prod.demand += amount;
        mat.demand += amount * prod.types.material_amount.1;
        mat.calculate_scarcity();
        if amount <= prod.supply {
            prod.supply -= amount;
            prod.demand -= amount;
            return Ok(true)
        } else {
            if mat.scarcity > 50 || mat.supply < (amount * prod.types.material_amount.1)
                { mat.demand -= amount * prod.types.material_amount.1;
                    return Err("Material scarce.");
                }
            { //for now we immediately produce product and deliver it
                manufacture_product(prod, mat, &amount);
                prod.supply -= amount;
                prod.demand -= amount;
            }
            return Ok(false)
        }

    }

    //pub fn is_in_supply() {}

    #[no_mangle]
    pub extern fn update_supply(&mut self, name: &str, amount: usize) -> bool {
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
    pub extern fn get_product_count (&self) -> usize {
        self.products.len()
    }

    #[no_mangle]
    pub extern fn get_product_types (&self, name: &str) -> &ProductType {
        &self.products.get(name).unwrap().types
    }

    #[no_mangle]
    pub extern fn tst_set_product_supply(&mut self, name: &str, count: usize) {
        self.products.get_mut(name).unwrap().supply = count;
    }

    #[no_mangle]
    pub extern fn tst_get_material_params(&self, name: &str) -> &Material {
        self.materials.get(name).unwrap()
    }

    #[no_mangle]
    pub extern fn tst_get_materials(&self) -> &HashMap<String, Material> {
        &self.materials
    }

    #[no_mangle]
    pub extern fn tst_get_products(&self) -> &HashMap<String, Product> {
        &self.products
    }

}

fn manufacture_product(product: &mut Product, material: &mut Material, amount: &usize) {
    material.supply -= product.types.material_amount.1 * amount;
    material.demand -= product.types.material_amount.1 * amount;
    product.supply += amount;
    //product.demand -= count;
}

#[no_mangle]
pub extern fn init() -> Instance {
    Instance {
        materials: HashMap::new(),
        products: HashMap::new(),
    }
}

#[no_mangle]
pub extern fn load(materials: HashMap<String, Material>,
                   products: HashMap<String, Product>) -> Instance {
    Instance {
        materials,
        products,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_same_material() {
        let mut instance = init();
        instance.add_material(String::from("bla"), 8);
        instance.add_material(String::from("bla"), 1);
        assert_eq!(instance.materials.get("bla").unwrap().supply, 8);
    }

    #[test]
    fn add_product_without_material() {
        let mut instance = init();
        assert!(!instance.add_product(String::from("bla"), String::from("mat"), 5, 10));
    }

    #[test]
    fn add_same_product() {
        let mut instance = init();
        instance.add_material(String::from("bla"), 8);
        instance.add_product(String::from("bla"), String::from("bla"), 5, 10);
        instance.add_product(String::from("bla"), String::from("bla"), 0, 5);
        assert_eq!(instance.products.get("bla").unwrap().types.material_amount.1, 10);
    }
}