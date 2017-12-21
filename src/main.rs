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

extern crate resource_mng;
extern crate rand;

mod event_generator;

use std::{thread, time};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let cycles: usize;
    let millis: u64;
    if args.len() == 3 {
        cycles = args[1].parse().unwrap();
        millis = args[2].parse().unwrap();
    } else {
        cycles = 500;
        millis = 300;
    }
    let mut rng = rand::thread_rng();
    let mut instance = resource_mng::init();
    instance.add_material(String::from("first"), 10);
    let mut num: usize = 0;
    let mut f0_count: usize = 0;
    let mut f1_count: usize = 0;
    let mut f2_count: usize = 0;
    let mut f3_count: usize = 0;
    let time = time::Duration::from_millis(millis);

    while num < cycles {
        match event_generator::run(&mut instance, &mut rng, 512) {
            Some(result) => {
                match result.function_number {
                    0 => {
                        println!("[{}] Adding material \"{}\", supply: {}", num, result.name, result.amount);
                        f0_count +=1;
                    },
                    1 => {
                        println!("[{}] Adding product \"{}\" composed of {}x material \"{}\"", num, result.name, result.amount, result.material_id);
                        f1_count +=1;
                    },
                    2 => {
                        println!("[{}] Manufacturing product \"{}\"\
                    at the cost of material {}x \"{}\"",
                                 num, result.name, result.amount, result.material_id);
                        f2_count +=1;
                    },
                    3 => {
                        println!("[{}] Updating supply of material \"{}\" to {}", num, result.name, result.amount);
                        f3_count +=1;
                    },
                    _ => {}
                }
            }
            None => {}
        }
        num += 1;
        thread::sleep(time);
    }
    println!("\nProgramme ends at cycle {}.\n\
    f0 count: {}, f1 count: {}, f2 count: {}, f3 count: {}",
             num, f0_count, f1_count, f2_count, f3_count);

}