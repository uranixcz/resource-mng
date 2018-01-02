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
        cycles = 5000;
        millis = 0;
    }
    let mut rng = rand::thread_rng();
    let mut instance = resource_mng::init();
    instance.add_material(String::from("first"), 10);
    let mut num: usize = 0;
    let mut f0_count: usize = 0;
    let mut f1_count: usize = 0;
    let mut f2_count: usize = 0;
    let mut f3_count: usize = 0;
    let mut failed_scarce: usize = 0;
    let mut failed_no_supply: usize = 0;
    let time = time::Duration::from_millis(millis);
    let mut fn_num;
    let max_values: usize = 512;
    let mut evgen;

    while num < cycles || cycles == 0 {
        fn_num = rng.gen::<u8>() % 4;
        evgen = event_generator::run(&mut instance, &fn_num, &mut rng, &max_values);
        match fn_num {
            0 => {
                match evgen {
                    Ok(result) => {
                        println!("[{}] Adding material \"{}\" to the database, supply: {}",
                                 num, result.name, result.amount);
                        f0_count += 1;
                    },
                    Err(&1) => {
                        println!("[{}] Adding material failed. \
                        Name cannot be empty or contain only white spaces.", num);
                    },
                    Err(&2) => {
                        //println!("[{}] Adding material failed. \
                        //Supply cannot be zero.", num);
                    },
                    Err(&3) => {
                        //println!("[{}] Adding material failed. \
                        //Material already in database.", num);
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
                        println!("[{}] Adding product \"{}\" composed of {}x material \"{}\" \
                        to the database", num, result.name, result.amount, result.material_id);
                        f1_count +=1;
                    },
                    Err(&1) => {
                        //println!("[{}] Adding product failed. \
                        //Product name cannot be empty or contain only white spaces.", num);
                    }
                    Err(&2) => {
                        //println!("[{}] Adding product failed. \
                        //Material name cannot be empty or contain only white spaces.", num);
                    }
                    Err(&3) => {
                        //println!("[{}] Adding product failed. \
                        //Material amount required must not be zero.", num);
                    }
                    Err(&4) => {
                        println!("[{}] Adding product failed. \
                        Material does not exist.", num);
                    }
                    Err(&5) => {
                        //println!("[{}] Adding product failed. \
                        //Product already exists.", num);
                    }
                    Err(_) => {
                        println!("[{}] Adding product failed. Unknown error.", num);
                    }
                }
            },
            2 => {
                match evgen {
                    Ok(result) => {
                        //let tmp = instance.get_material_scarcity(&result.material_id);
                        match result.code {
                            &4 => {
                                println!("[{}] Manufacturing of {}x products \"{}\" DENIED. \
                        Material \"{}\" not available; scarcity: {}", num, result.amount, result.name, result.material_id,
                                         instance.get_material_scarcity(&result.material_id));
                                failed_no_supply +=1;
                            },
                            &5 => { println!("[{}] Manufacturing of {}x products \"{}\" DENIED. \
                        Material \"{}\" scarce: {} > 50.", num, result.amount, result.name, result.material_id,
                                           instance.get_material_scarcity(&result.material_id));
                                failed_scarce +=1;
                            },
                            &_ => println!("[{}] Manufacturing product \"{}\" \
                        at the cost of {}x material \"{}\", scarcity: {}",
                                           num, result.name, result.amount, result.material_id,
                                           instance.get_material_scarcity(&result.material_id)),
                        }
                        f2_count +=1;
                    },
                    Err(&2) => {
                        //println!("[{}] Manufacturing product failed. \
                        //Cannot order 0 products.", num);
                    },
                    Err(&3) => {
                        //println!("[{}] Manufacturing product failed. \
                        //No such material in database.", num);
                    },
                    Err(&4) => {
                        println!("[{}] Manufacturing product failed. \
                        Material not available.", num); //safeguard for future code changes
                        panic!("Material not available.");
                    },
                    Err(&5) => {
                        println!("[{}] Manufacturing product failed. \
                        Material scarce.", num); //safeguard for future code changes
                        panic!("Material scarce.");
                    },
                    Err(&6) => {
                        //println!("[{}] Manufacturing product failed. \
                        //Product database is empty.", num);
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
                        println!("[{}] Updating supply of material \"{}\" to {}; \
                        demand: {}, scarcity: {}", num, result.name, result.amount,
                                 instance.get_material_demand(&result.name),
                                 instance.tst_get_material(&result.name).calculate_scarcity()
                                 );
                        f3_count +=1;
                    },
                    Err(&1) => {
                        println!("[{}] Updating supply of material failed. \
                        No materials in database.", num);
                    },
                    Err(&2) => {
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
        if millis != 0 { thread::sleep(time); }
    }
    println!("\nProgram ends at cycle {}.\n\
    Functions passed | Add material: {}, Add product: {}, Order product: {}, Update supply: {}",
             num, f0_count, f1_count, f2_count, f3_count);
    println!("Failed orders    | no supply: {}, scarce: {}", failed_no_supply, failed_scarce);

}