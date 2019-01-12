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

extern crate resource_mng;
extern crate rand;

mod event_generator;

use std::{thread, time};
use std::env;
use rand::Rng;
use resource_mng::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    let cycles: usize;
    let millis: u64;
    if args.len() == 3 {
        cycles = args[1].parse().unwrap();
        millis = args[2].parse().unwrap();
    } else {
        cycles = 500;
        millis = 0;
    }
    let mut rng = rand::thread_rng();
    let mut instance = init();
    instance.verbose = true;
    let mut instance = &mut instance;
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

    println!("Generating initial database entries, please wait...");
    event_generator::init(instance, &mut rng, &max_values, &cycles);

    while num < cycles || cycles == 0 {
        fn_num = rng.gen::<u8>() % 10;
        evgen = event_generator::run(&mut instance, fn_num, &mut rng, &max_values);
        match fn_num {
            0 => {
                match evgen {
                    Ok(result) => {
                        println!("[{}] Adding material #{} to the database, supply: {}",
                                 num, result.primary_id, result.amount);
                        f0_count += 1;
                    },
                    Err(1) => {
                        println!("[{}] Adding material failed. \
                        Name cannot be empty or contain only white spaces.", num);
                    },
                    Err(2) => {
                        //println!("[{}] Adding material failed. \
                        //Supply cannot be zero.", num);
                    },
                    Err(3) => {
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
                        println!("[{}] Adding product #{} composed of {}x material #{} \
                        to the database", num, result.primary_id, result.amount, result.secondary_id);
                        f1_count +=1;
                    },
                    Err(1) => {
                        //println!("[{}] Adding product failed. \
                        //Product name cannot be empty or contain only white spaces.", num);
                    }
                    Err(2) => {
                        //println!("[{}] Adding product failed. \
                        //Material name cannot be empty or contain only white spaces.", num);
                    }
                    Err(3) => {
                        //println!("[{}] Adding product failed. \
                        //Material amount required must not be zero.", num);
                    }
                    Err(4) => {
                        println!("[{}] Adding product failed. \
                        Material does not exist.", num);
                    }
                    Err(5) => {
                        //println!("[{}] Adding product failed. \
                        //Product already exists.", num);
                    }
                    Err(_) => {
                        println!("[{}] Adding product failed. Unknown error.", num);
                    }
                }
            },
            2|3|4|5 => {
                match evgen {
                    Ok(result) => {
                        //let tmp = instance.get_material_scarcity(&result.material_id);
                        match result.code {
                            &4 => {
                                println!("[{}] Manufacturing of {}x product #{} DENIED. \
                        Material #{} not available; scarcity: {}", num, result.amount, result.primary_id, result.secondary_id,
                                         get_material_scarcity(instance, &result.secondary_id));
                                failed_no_supply +=1;
                            },
                            &5 => { println!("[{}] Manufacturing of {}x product #{} DENIED. \
                        Material #{} scarce: {} > 50.", num, result.amount, result.primary_id, result.secondary_id,
                                             get_material_scarcity(instance, &result.secondary_id));
                                failed_scarce +=1;
                            },
                            &_ => println!("[{}] Manufacturing product #{} complete \
                        at the cost of {}x material #{}; scarcity: {}",
                                           num, result.primary_id, result.amount, result.secondary_id,
                                           get_material_scarcity(instance, &result.secondary_id)),
                        }
                        f2_count +=1;
                    },
                    Err(2) => {
                        //println!("[{}] Manufacturing product failed. \
                        //Cannot order 0 products.", num);
                    },
                    Err(3) => {
                        //println!("[{}] Manufacturing product failed. \
                        //No such material in database.", num);
                    },
                    Err(4) => {
                        println!("[{}] Manufacturing product failed. \
                        Material not available.", num); //safeguard for future code changes
                        panic!("Material not available.");
                    },
                    Err(5) => {
                        println!("[{}] Manufacturing product failed. \
                        Material scarce.", num); //safeguard for future code changes
                        panic!("Material scarce.");
                    },
                    Err(6) => {
                        //println!("[{}] Manufacturing product failed. \
                        //Product database is empty.", num);
                    },
                    Err(_) => {
                        println!("[{}] Manufacturing product failed. \
                        Unknown error.", num);
                    }
                }
            },
            6|7 => {
                match evgen {
                    Ok(result) => {
                        println!("[{}] Adding new variant to product #{} \
                        consisting of material #{}.", num, result.primary_id, result.secondary_id);
                    },
                    Err(_) => {
                        println!("[{}] Adding new variant failed. No such product or material.", num);
                    }
                }
            }
            8|9 => {
                match evgen {
                    Ok(result) => {
                        println!("[{}] Updating supply of material #{} to {}; \
                        demand: {}, scarcity: {}", num, result.primary_id, result.amount,
                                 get_material_demand(instance, &result.primary_id),
                                 tst_get_material(instance, &result.primary_id).get_scarcity()
                                 );
                        f3_count +=1;
                    },
                    Err(1) => {
                        println!("[{}] Updating supply of material failed. \
                        No materials in database.", num);
                    },
                    Err(2) => {
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
    eprintln!("\nProgram ends at cycle {}.\n\
    Functions passed | Add material: {}, Add product: {}, Order product: {}, Update supply: {}",
             num, f0_count, f1_count, f2_count, f3_count);
    eprintln!("Failed orders    | no supply: {}, scarce: {}", failed_no_supply, failed_scarce);
}