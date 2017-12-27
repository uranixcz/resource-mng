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
use rand::Rng;

fn main() {
    let args: Vec<String> = env::args().collect();
    let cycles: usize;
    let millis: u64;
    if args.len() == 3 {
        cycles = args[1].parse().unwrap();
        millis = args[2].parse().unwrap();
    } else {
        cycles = 500;
        millis = 3000;
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
    let mut fn_num;
    let max_values: usize = 512;
    let mut evgen;

    while num < cycles {
        fn_num = rng.gen::<u8>() % 4;
        evgen = event_generator::run(&mut instance, &fn_num, &mut rng, &max_values);
        match fn_num {
            0 => {
                match evgen {
                    Ok(result) => {
                        println!("[{}] Adding material \"{}\", supply: {}",
                                 num, result.name, result.amount);
                        f0_count += 1;
                    },
                    Err(&1) => {
                        println!("[{}] Adding material failed. \
                        Name cannot be empty or contain only white spaces.", num);
                    },
                    Err(&2) => {
                        println!("[{}] Adding material failed. \
                        Supply cannot be zero.", num);
                    },
                    Err(&3) => {
                        println!("[{}] Adding material failed. \
                        Material already in database.", num);
                    },
                    Err(_) => {
                        println!("[{}] Adding material failed. \
                        Unknown error.", num);
                    }
                }
            },
            1 => {
                match evgen {
                    Ok(result) => {
                        println!("[{}] Adding product \"{}\" composed of {}x material \"{}\"",
                                 num, result.name, result.amount, result.material_id);
                        f1_count +=1;
                    },
                    Err(_) => { //fix me
                        println!("[{}] Adding product failed. Could not add product.", num);
                    }
                }
            },
            2 => {
                match evgen {
                    Ok(result) => {
                        println!("[{}] Manufacturing product \"{} \"\
                    at the cost of material {}x \"{}\"",
                                 num, result.name, result.amount, result.material_id);
                        f2_count +=1;
                    },
                    Err(&1) => {
                        println!("[{}] Manufacturing product failed. \
                        Product count cannot be zero.", num);
                    },
                    Err(_) => {
                        println!("[{}] Manufacturing product failed. \
                        Unknown error.", num);
                    }
                }
            },
            3 => {
                match evgen {
                    Ok(result) => {
                        println!("[{}] Updating supply of material \"{}\" to {}",
                                 num, result.name, result.amount);
                        f3_count +=1;
                    },
                    Err(&1) => {
                        println!("[{}] Updating supply of material failed. \
                        No materials in database.", num);
                    },
                    Err(&2) => { //fix me never happens
                        println!("[{}] Updating supply of material failed. \
                        Supply update failed.", num);
                    },
                    Err(_) => {
                        println!("[{}] Updating supply of material failed. \
                        Unknown error.", num);
                    }
                }
            },
            _ => { panic!("Out of range.")}
        }

        num += 1;
        thread::sleep(time);
    }
    println!("\nProgramme ends at cycle {}.\n\
    Pass counts for fn Add material: {}, Add product: {}, Order product: {}, Update supply: {}",
             num, f0_count, f1_count, f2_count, f3_count);

}