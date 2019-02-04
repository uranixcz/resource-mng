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

use rand::Rng;
use rand::ThreadRng;
use resource_mng::*;

pub struct RunResult {
    pub code: &'static u8,
    pub primary_id: usize,
    pub amount: usize,
    pub secondary_id: usize,
}

pub fn run(instance: &mut Instance, fn_num: u8, rng: &mut ThreadRng, max_values: &usize) -> Result<RunResult, u8> {
    match fn_num {
        0 => { //add material
            let id = rng.gen::<u16>() as usize;
            let supply = rng.gen::<usize>() % max_values;
            match add_material(instance, id, supply) {
                0 => {
                    Ok(RunResult {
                        code: &0,
                        primary_id: id,
                        amount: supply,
                        secondary_id: 999999998 //should not be displayed
                    })},
                num => { Err(num) },
            }
        }
        1 => { // add product
            let id = rng.gen::<u16>() as usize;
            let material_amount = rng.gen::<usize>() % max_values /32;
            let rnd_index = rng.gen::<usize>() % get_material_count(instance);
            let priority = rng.gen::<usize>() % 4;
            let material_id = tst_get_materials(instance).iter().enumerate()
                .nth(rnd_index)
                .unwrap()
                .1
                .0.clone();
            //let work_complexity = rng.gen::<u8>();
            match add_product(instance, id, material_id.clone(),
                                       material_amount, priority) {
                0 => Ok(RunResult {
                    code: &0,
                    primary_id: id,
                    amount: material_amount,
                    secondary_id: material_id
                }),
                num => { Err(num) } //"Could not add product"
            }
        }
        2|3|4|5 => { // order product
            let amount = rng.gen::<usize>() % max_values /48;
            let product_count = get_product_count(instance);
            let rnd_index = if product_count > 0 {
                rng.gen::<usize>() % product_count
            } else { return Err(6) }; //"Product database is empty."
            let id;
            let tmp;
            {let product_item = tst_get_products(instance).iter().enumerate()
                .nth(rnd_index)
                .unwrap().1;
                id = product_item.0.clone();
                tmp = product_item.1.get_variant(0).components.clone();
            }
            let tmp1; let tmp2;
            {//let tmp = product_item.1.get_variant(0).material_and_amount.clone();
            tmp1 = tmp.material_id.clone();
            tmp2 = tmp.material_amount;} //fix me
            match order_product(instance, id, amount, 0, 0, true) {
                1 => { //&0 not active atm
                    //let (tmp, tmp1) = instance.get_product_types(&name).material_amount.clone_into();
                    Ok(RunResult {
                        code: &1,
                        primary_id: id.clone(),
                        amount: amount * tmp2,
                        secondary_id: tmp1,
                    })},
                4 => {
                    Ok(RunResult {
                        code: &4,
                        primary_id: id.clone(),
                        amount,
                        secondary_id: tmp1,
                    })},
                5 => {
                    Ok(RunResult {
                        code: &5,
                        primary_id: id.clone(),
                        amount,
                        secondary_id: tmp1,
                    })},
                num => {
                    Err(num)
                },
            }

        }
        6|7 => { // add product variant
            let product_count = get_product_count(instance);
            let rnd_index = if product_count > 0 {
                rng.gen::<usize>() % product_count
            } else { return Err(6) }; //"Product database is empty."
            let id = tst_get_products(instance).iter().enumerate()
                .nth(rnd_index)
                .unwrap()
                .1
                .0.clone();

            let material_count = get_material_count(instance);
            let rnd_index = if material_count > 0 {
                rng.gen::<usize>() % material_count
            } else { return Err(6) }; //"Material database is empty."
            let material_id = tst_get_materials(instance).iter().enumerate()
                .nth(rnd_index)
                .unwrap()
                .1
                .0.clone();

            let material_amount = rng.gen::<usize>() % max_values /32;

            match add_product_variant(instance, id, material_id, material_amount) {
                0 => Ok(RunResult {
                    code: &0,
                    primary_id: id,
                    amount: 0,
                    secondary_id: material_id
                }),
                num => Err(num), //"Could not add product"
            }
        }
        8|9 => { // update supply
            let amount = rng.gen::<usize>() % max_values;
            let material_count = get_material_count(instance);
            let rnd_index = if material_count > 0 {
                rng.gen::<usize>() % material_count
            } else { return Err(1) }; //"No materials in database."
            let id = tst_get_materials(instance).iter().enumerate()
                .nth(rnd_index)
                .unwrap()
                .1
                .0.clone();
            if update_supply(instance, id, amount) {
                Ok(RunResult {
                    code: &0,
                    primary_id: id,
                    amount,
                    secondary_id: 999999999, //should not be displayed
                })
            } else { Err(2) } //"Supply update failed"
        }
        _ => Err(255) //"No such function"
    }
}

pub fn init(instance: &mut Instance, rng: &mut ThreadRng, max_values: &usize, cycles: &usize) {
    add_material(instance, 1, 10);
    let tmp = rng.gen::<usize>() % cycles;
    //println!("{}, {}", tmp, cycles);
    let max: usize = if *cycles > 10 { tmp } else { 10 };
    let mut cnt: usize = 0;
    while cnt < max {
        if rng.gen::<u8>() % 2 == 0 {
            let name = rng.gen::<u16>() as usize;
            let supply = rng.gen::<usize>() % max_values;
            add_material(instance, name, supply);
        } else {
            let name = rng.gen::<u16>() as usize;
            let material_amount = rng.gen::<usize>() % max_values /32;
            let rnd_index = rng.gen::<usize>() % get_material_count(instance);
            let priority = rng.gen::<usize>() % 4;
            let material_id = tst_get_materials(instance).iter().enumerate()
                .nth(rnd_index)
                .unwrap()
                .1
                .0.clone();
            //let work_complexity = rng.gen::<u8>();
            add_product(instance, name, material_id.clone(),
                                       material_amount, priority);
        }
        cnt += 1;
    }
}