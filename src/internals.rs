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
use crate::{Order, Product, ProductVariant, Material, PRIORITIES};
use std::cmp::Ordering;

impl ProductVariant {
    fn get_production_efficiency(&self) -> f64 {
        self.components.scarcity_cache as f64 / self.work_complexity
    }
}

impl PartialOrd for ProductVariant {
    fn partial_cmp(&self, other: &ProductVariant) -> Option<Ordering> {
        let my = self.get_production_efficiency();
        let other = other.get_production_efficiency();
        my.partial_cmp(&other)
    }
}

impl PartialEq for ProductVariant {
    fn eq(&self, other: &ProductVariant) -> bool {
        let my = self.get_production_efficiency();
        let other = other.get_production_efficiency();
        my == other
    }
}

pub fn process_queue(production_queue: &mut [Vec<Order>; PRIORITIES],
                     products: &mut HashMap<usize, Product>,
                     materials: &mut HashMap<usize, Material>,
                     finished_products: &mut Vec<Order>,
                     verbose: usize)
{
    for q in production_queue.iter_mut() {
        let mut i: usize = 0;
        //let mut to_remove = Vec::new();
        while i != q.len() {
            let mut found = false;
            let q_product = products.get_mut(&q[i].product_id).unwrap();

            // update scarcity cache for components; better solution wanted
            for variant in q_product.variants.iter_mut() {
                let variant_material = materials.get_mut(&variant.components.material_id).unwrap();
                variant_material.scarcity_cache = variant_material.get_scarcity();
                variant.components.scarcity_cache = variant_material.scarcity_cache;
            }

            // sort variants by efficiency except the preferred one
            if q_product.variants.len() > 1 {
                if verbose >= crate::VERBOSITY_INNER {
                    if cfg!(feature = "cz") {
                        println!(" * Kalkuluji nejefektivnější variantu produktu #{} pro výrobu.",
                                 q[i].product_id);
                    } else {
                        println!(" * Calculating the most efficient variant of product #{} for production.",
                                 q[i].product_id);
                    }
                }
                let index = q_product.variants.iter().position(|x| x.id == q[i].preferred_variant).unwrap();
                let swap = q_product.variants.remove(index);
                if q_product.variants.len() > 1 { q_product.variants.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap()); }
                q_product.variants.insert(0, swap);
            }

            // manufacture the first one to meet conditions
            for variant in q_product.variants.clone() {
                let q_material = materials.get_mut(&variant.components.material_id).unwrap();
                let material_amount = q[i].product_amount * variant.components.material_amount;
                if q_material.supply >= material_amount {
                    if variant.id != q[i].preferred_variant { q_material.demand += material_amount; }
                    q_product.manufacture(q_material, q[i].product_amount, &variant);
                    q_product.deliver(q[i].product_amount);
                    let finished_product = q.remove(i);
                    if verbose >= crate::VERBOSITY_INNER {
                        if cfg!(feature = "cz") {
                            println!(" * Vyrábím {}x produkt #{}, varianta #{} (preferovaná byla {}) z fronty priority {}.",
                                     finished_product.product_amount, finished_product.product_id, variant.id, finished_product.preferred_variant, q_product.priority + 1);
                        } else {
                            println!(" * Manufacturing {}x product #{}, variant #{} (preferred was {}) from priority {} production queue.",
                                     finished_product.product_amount, finished_product.product_id, variant.id, finished_product.preferred_variant, q_product.priority + 1);
                        }
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