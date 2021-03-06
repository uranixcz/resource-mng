/*
* Copyright 2017-2019 Michal Mauser
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

const PRIORITIES: usize = 4;
const EQUILIBRIUM: f64 = 50.0;
pub const VERBOSITY_QUIET: usize = 0;
pub const VERBOSITY_RESULTS: usize = 1; // not used in the lib
pub const VERBOSITY_INNER: usize = 2;
pub const VERBOSITY_FAILURES: usize = 3; // not used in the lib; should be replaced by callbacks

//#[derive(Debug)]
pub struct Product {
    //name: String,
    pub variants: Vec<ProductVariant>,
    //scarcity: usize,
    pub supply: f64,
    pub demand: f64,
    pub priority: usize,
}

impl Product {
    fn manufacture(&mut self, material: &mut Material, amount: f64, variant: &ProductVariant) {
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

    fn deliver(&mut self, amount: f64) {
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

#[repr(C)]
#[derive(Copy, Clone, PartialEq)]
pub struct Component {
    pub material_id: usize,
    pub material_amount: f64,
    pub scarcity_cache: f64,
}

#[derive(Debug)]
#[derive(Copy, Clone)]
pub struct Material {
    pub scarcity_cache: f64,
    pub demand: f64,
    pub supply: f64,
    //deposit_size: usize,
}

impl Material {
    #[must_use]
    pub extern fn get_scarcity(&self) -> f64 { //100/2=50
        if self.supply != 0.0 {
            self.demand * EQUILIBRIUM / (self.supply /*+ self.deposit_size*/)
        } else { std::f64::INFINITY }
    }
}

#[repr(C)]
pub struct Order {
    product_id: usize,
    product_amount: f64,
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
    materials: Vec<Material>,
    products: Vec<Product>,
    production_queue: [Vec<Order>; PRIORITIES],
    finished_products: Vec<Order>,
    pub verbose: usize,
}

impl Instance {
    pub fn get_product(&self, id: usize) -> &Product {
        &self.products[id]
    }

    pub fn get_products(&self) -> &Vec<Product> {
        &self.products
    }

    pub fn get_material(&self, id: usize) -> &Material {
        &self.materials[id]
    }

    pub fn get_materials(&self) -> &Vec<Material> {
        &self.materials
    }
}

#[no_mangle]
pub extern fn init() -> Box<Instance> {
    Box::from(Instance {
        materials: Vec::new(),
        products: Vec::new(),
        production_queue: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
        finished_products: Vec::new(),
        verbose: 0,
    })
}

#[no_mangle]
pub extern fn add_material(instance: &mut Instance, supply: f64) -> u8 {
    const ZERO_SUPPLY: u8 = 2;
    //const DUPLICATE_MATERIAL: u8 = 3;

    if supply <= 0.0 { return ZERO_SUPPLY; }

    instance.materials.push(Material{
            scarcity_cache: 0.0,
            demand: 0.0,
            supply,
        });
        0 //ok
}

#[no_mangle]
pub extern fn add_product(instance: &mut Instance, material_id: usize, material_amount: f64, priority: usize, work_complexity: f64) -> u8 {
    const ZERO_MATERIAL: u8 = 3;
    const NO_SUCH_MATERIAL: u8 = 4;
    //const DUPLICATE_PRODUCT: u8 = 5;

    if material_amount <= 0.0 { return ZERO_MATERIAL; }
    if instance.materials.len() <= material_id { return NO_SUCH_MATERIAL; }
    instance.products.push(Product {
        //name,
        variants: vec![ProductVariant {
            id: 0,
            components: Component { material_id, material_amount, scarcity_cache: 0.0 },
            work_complexity,
        }],
        supply: 0.0,
        demand: 0.0,
        priority,
    });
    0
}

#[no_mangle]
pub extern fn order_product(instance: &mut Instance,
                            id: usize,
                            amount: f64,
                            variant_id: usize,
                            user_id: usize,
                            allow_substitution: bool) -> u8
{
    const OK_QUEUE: u8 = 1;
    const CANNOT_ORDER_0_PRODUCTS: u8 = 2;
    const NO_SUCH_MATERIAL: u8 = 3;
    const MATERIAL_NOT_AVAIL: u8 = 4;
    const MATERIAL_SCARCE: u8 = 5;

    if amount <= 0.0 { return CANNOT_ORDER_0_PRODUCTS; }
    let products = &mut instance.products;
    if products.is_empty() { panic!("no products in database"); }
    let mut prod = products.remove(id);

    let variant = *prod.get_variant(variant_id);
    let production_queue = &mut instance.production_queue;
    let mut material;
    if instance.materials.len() > variant.components.material_id {
        material = instance.materials.remove(variant.components.material_id)
    } else { return NO_SUCH_MATERIAL }
    /*let mut material = match instance.materials.remove(variant.components.material_id) {
        Some(m) => m,
        None => return NO_SUCH_MATERIAL, //No such material in database.
    };*/
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

    if code > OK_QUEUE {
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
pub extern fn update_supply(instance: &mut Instance, id: usize, amount: f64) -> bool {
    let result = match instance.materials.get_mut(id) {
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
pub extern fn add_product_variant(instance: &mut Instance, product_id: usize, material_id: usize, material_amount: f64, work_complexity: f64) -> u8 {
    const NO_SUCH_PRODUCT: u8 = 1;
    const NO_SUCH_MATERIAL: u8 = 2;

    if instance.products.len() <= product_id { return NO_SUCH_PRODUCT; }
    if instance.materials.len() <= material_id { return NO_SUCH_MATERIAL; }
    let product = instance.products.get_mut(product_id).unwrap();
    let variant_id = product.variants.len(); //this must be changed when remove_product_variant is implemented!
    product.variants.push(ProductVariant {
        id: variant_id,
        components: Component { material_id, material_amount, scarcity_cache: 0.0 },
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
pub extern fn get_material_demand(instance: &Instance, id: usize) -> f64 {
    instance.materials[id].demand
}

#[no_mangle]
pub extern fn get_material_supply(instance: &Instance, id: usize) -> f64 {
    instance.materials[id].supply
}

#[no_mangle]
pub extern fn get_material_scarcity(instance: &Instance, id: usize) -> f64 {
    instance.materials.get(id).unwrap().scarcity_cache
}

#[no_mangle]
pub extern fn get_product_count(instance: &Instance) -> usize {
    instance.products.len()
}

#[no_mangle]
pub extern fn get_product_supply(instance: &Instance, id: usize) -> f64 {
    instance.products[id].supply
}

#[no_mangle]
pub extern fn get_product_demand(instance: &Instance, id: usize) -> f64 {
    instance.products[id].demand
}

#[no_mangle]
pub extern fn get_product_priority(instance: &Instance, id: usize) -> usize {
    instance.products[id].priority
}

#[no_mangle]
pub extern fn get_product_variant(instance: &Instance, product_id: usize, variant_id: usize) -> Component {
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
                product_amount: 0.0,
                preferred_variant: 0,
                user_id: 0,
                allow_substitution: false,
            },
        }
    }
}

#[no_mangle]
pub extern fn get_finished_count(instance: &Instance) -> usize {
    instance.finished_products.len()
}

#[no_mangle]
pub extern fn get_queue_len(instance: &Instance) -> usize {
    let mut total = 0;
    for i in instance.production_queue.iter() {
        total += i.len()
    }
    total
}

pub fn get_product_variants(instance: &Instance, id: usize) -> &Vec<ProductVariant> {
    &instance.products[id].variants
}

#[no_mangle]
pub extern fn tst_set_product_supply(instance: &mut Instance, id: usize, count: f64) {
    instance.products.get_mut(id).unwrap().supply = count;
}

#[deprecated(since="0.1.6", note="please use `self.get_material` instead")]
pub fn tst_get_material(instance: &Instance, id: usize) -> Material {
    instance.materials[id].clone()
}

#[deprecated(since="0.1.6", note="please use `self.get_materials` instead")]
pub fn tst_get_materials(instance: &Instance) -> &Vec<Material> {
    &instance.materials
}

#[deprecated(since="0.1.6", note="please use `self.get_products` instead")]
pub fn tst_get_products(instance: &Instance) -> &Vec<Product> {
    &instance.products
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_same_material() {
        let instance = &mut init();
        add_material(instance, 8.);
        add_material(instance, 1.);
        assert_eq!(instance.materials.get(0).unwrap().supply, 8.);
    }

    #[test]
    fn add_product_without_material() {
        let instance = &mut init();
        assert_ne!(!add_product(instance,12345, 10., 0, 1.0), 0);
    }

    #[test]
    fn add_same_product() {
        let instance = &mut init();
        add_material(instance, 8.);
        add_product(instance, 0, 10., 0, 1.0);
        add_product(instance, 0, 5., 0, 1.0);
        assert_eq!(instance.products.get(0).unwrap().variants.first().unwrap().components.material_amount, 10.);
    }

    #[test]
    fn add_prod_zero_mat() {
        let instance = &mut init();
        add_material(instance, 8.);
        assert_ne!(add_product(instance, 0, 0., 0, 1.0), 0);
    }

    #[test]
    fn order_zero_products() {
        let instance = &mut init();
        assert_eq!(order_product(instance, 0, 0., 0, 0, true), 2);
    }
}