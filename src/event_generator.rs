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

use rand::Rng;
use rand::ThreadRng;
use resource_mng::Instance;

pub struct RunResult {
    pub name: String,
    pub amount: usize,
    pub material_id: String,
}

pub fn run(instance: &mut Instance, fn_num: &u8, rng: &mut ThreadRng, max_values: &usize) -> Result<RunResult, &'static u8> {
    match fn_num {
        &0 => { //add material
            let name = rng.gen::<u16>().to_string();
            let supply = rng.gen::<usize>() % max_values;
            match instance.add_material(name.clone(), supply) {
                0 => {
                Ok(RunResult {
                    name,
                    amount: supply,
                    material_id: String::new()
                })},
                1 => { Err(&1) },
                2 => { Err(&2)},
                3 => { Err(&3)},
                _ => { Err(&255)},
            }
        }
        &1 => { // add product
            let name = rng.gen::<u16>().to_string();
            let material_amount = rng.gen::<usize>() % max_values /64;
            let rnd_index = rng.gen::<usize>() % instance.get_material_count();
            let material_id = instance.tst_get_materials().iter().enumerate()
                .nth(rnd_index)
                .unwrap()
                .1
                .0.clone();
            let work_complexity = rng.gen::<u8>();
            if instance.add_product(name.clone(), material_id.clone(), work_complexity, material_amount) {
                Ok(RunResult {
                    name,
                    amount: material_amount,
                    material_id
                })
            } else { Err(&1) } //"Could not add product"
        }
        &2 => { // order product
            let amount = rng.gen::<usize>() % max_values /64;
            let product_count = instance.get_product_count();
            let rnd_index = if product_count > 0 {
                rng.gen::<usize>() % product_count
            } else { return Err(&1) }; //"Product count cannot be zero"
            let name = instance.tst_get_products().iter().enumerate()
                .nth(rnd_index)
                .unwrap()
                .1
                .0.clone();
            let tmp1; let tmp2;
            {let tmp = &instance.get_product_types(&name).material_amount;
            tmp1 = tmp.0.clone();
            tmp2 = tmp.1;} //fix me
            match instance.order_product(&name, amount) {
                Ok(_) => {
                    //let (tmp, tmp1) = instance.get_product_types(&name).material_amount.clone_into();
                    Ok(RunResult {
                    name: name.clone(),
                    amount: amount * tmp2,
                    material_id: tmp1,
                })},
                Err(_) => {
                    Err(&255)
                },
            }
        }
        &3 => { // update supply
            let amount = rng.gen::<usize>() % max_values;
            let material_count = instance.get_material_count();
            let rnd_index = if material_count > 0 {
                rng.gen::<usize>() % material_count
            } else { return Err(&1) }; //"No materials in database."
            let name = instance.tst_get_materials().iter().enumerate()
                .nth(rnd_index)
                .unwrap()
                .1
                .0.clone();
            if instance.update_supply(&name, amount) {
                Ok(RunResult {
                    name,
                    amount,
                    material_id: String::new(),
                })
            } else { Err(&2) } //"Supply update failed"
        }
        _ => Err(&255) //"No such function"
    }
}