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
use crate::{Order, Product, Material, PRIORITIES};

pub fn process_queue(production_queue: &mut [Vec<Order>; PRIORITIES],
                     products: &mut HashMap<usize,Product>,
                     materials: &mut HashMap<usize,Material>,
                     finished_products: &mut Vec<Order>,
                     verbose: bool)
{
    for q in production_queue.iter_mut() {
        let mut i:usize = 0;
        //let mut to_remove = Vec::new();
        while i != q.len() {
            let mut found = false;
            let q_product = products.get_mut(&q[i].product_id).unwrap();

            for variant in q_product.variants.iter_mut() {
                let variant_material = materials.get_mut(&variant.components.material_id).unwrap();
                variant_material.scarcity_cache = variant_material.get_scarcity();
                variant.components.scarcity_cache = variant_material.scarcity_cache;
            }
            if q_product.variants.len() >= 2 {
                let index = q_product.variants.iter().position(|x| x.id == q[i].preferred_variant).unwrap();
                let swap = q_product.variants.remove(index);
                if q_product.variants.len() >= 2 { q_product.variants.sort_unstable(); }
                q_product.variants.insert(0, swap);
            }

            for variant in q_product.variants.clone() {
                let q_material = materials.get_mut(&variant.components.material_id).unwrap();
                let material_amount = q[i].product_amount * variant.components.material_amount;
                if q_material.supply >= material_amount {
                    if variant.id != q[i].preferred_variant {q_material.demand += material_amount;}
                    q_product.manufacture(q_material, q[i].product_amount, &variant);
                    q_product.deliver(q[i].product_amount);
                    let finished_product = q.remove(i);
                    if verbose {
                        println!(" * Manufacturing {}x product #{}, variant {} from priority {} production queue.",
                                 finished_product.product_amount, finished_product.product_id, variant.id, q_product.priority+1);
                    }
                    finished_products.push(finished_product);
                    found = true;
                    break;
                }
            }
            if !found { i += 1; }
        }
    }
}