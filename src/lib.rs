#[cfg(test)]
mod tests;

use std::collections::HashMap;

struct Product {
    //name: String,
    material_id: String,
    material_amount: usize,
    work_complexity: u8,
    //scarcity: usize,
    supply: usize,
    demand: usize,
}

struct Material {
    //name: String,
    scarcity: usize,
    demand: usize,
    supply: usize,
    //deposit_size: usize,
}

impl Material {
    fn calculate_scarcity(&mut self) {
        self.scarcity = self.demand * 100 / (self.supply /*+ self.deposit_size*/) / 2;
    }
}

pub struct Instance {
    materials: HashMap<String,Material>,
    products: HashMap<String,Product>,
}

impl Instance {
    pub fn add_material(&mut self, name: String, supply: usize) {
        self.materials.insert(name,Material{
            //name,
            scarcity: 0,
            demand: 0,
            supply,
            //deposit_size,
        });
    }

    pub fn add_product(&mut self, name: String, material_id: String, work_complexity: u8, material_count: usize) {
        self.products.insert(name, Product{
            //name,
            material_id,
            work_complexity,
            supply: 0,
            demand: 0,
            material_amount: material_count
        });
    }

    pub fn demand_product(&mut self, name: &str, count: usize) -> Result<bool,&str> {
        let prod = self.products.get_mut(name).unwrap();
        let mat = match self.materials.get_mut(&prod.material_id) {
            Some(m) => m,
            None => return Err("No such material in database."),
        };
        prod.demand += count;
        mat.demand += count * prod.material_amount;
        mat.calculate_scarcity();
        if count <= prod.supply {
            prod.supply -= count;
            prod.demand -= count;
            return Ok(true)
        } else {
            if mat.scarcity > 50 || mat.supply < (count * prod.material_amount) { return Err("Materials scarce.") }
            { //for now we immediately produce product and deliver it
                manufacture_product(prod, mat, &count);
                prod.supply -= count;
                prod.demand -= count;
            }
            return Ok(false)
        }

    }

    //pub fn is_in_supply() {}

    //pub fn update_supply() {}

    //pub fn update_material_deposit_size() {}


    /*fn search_material(&mut self, name: &str) -> Option<&mut Material> {
        for mat in self.materials.iter_mut() {
            if mat.name == name {
                return Some(mat)
            }
        }
        None
    }*/

    pub fn tst_set_product_supply(&mut self, name: &str, count: usize) {
        self.products.get_mut(name).unwrap().supply = count;
    }

}

/*fn search_product<'products>(products: &'products Vec<Product>, name: &str) -> Option<&'products Product> {
    for prod in products.iter() {
        if prod.name == name {
            return Some(prod)
        }
    }
    None
}

fn search_material<'materials>(materials: &'materials mut Vec<Material>, name: &str) -> Option<&'materials mut Material> {
    for mat in materials.iter_mut() {
        if mat.name == name {
            return Some(mat)
        }
    }
    None
}*/

fn manufacture_product(product: &mut Product, material: &mut Material, count: &usize) {
    material.supply -= product.material_amount * count;
    material.demand -= product.material_amount * count;
    product.supply += count;
    //product.demand -= count;
}

pub fn init() -> Instance {
    Instance {
        materials: HashMap::new(),
        products: HashMap::new(),
    }
}
