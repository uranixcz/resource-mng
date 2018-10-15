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
    instance.verbose = false;
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

    println!("Generuji úvodní položky databáze, prosím čekejte...");
    event_generator::init(instance, &mut rng, &max_values, &cycles);

    while num < cycles || cycles == 0 {
        fn_num = rng.gen::<u8>() % 10;
        evgen = event_generator::run(&mut instance, &fn_num, &mut rng, &max_values);
        match fn_num {
            0 => {
                match evgen {
                    Ok(result) => {
                        println!("[{}] Přidávám materiál \"{}\" do databáze; množství: {}",
                                 num, result.name, result.amount);
                        f0_count += 1;
                    },
                    Err(&1) => {
                        println!("[{}] Přidání materiálu selhalo. \
                        Název nesmí být prázdný nebo obsahovat netiskuté znaky.", num);
                    },
                    Err(&2) => {
                        //println!("[{}] Přidání materiálu selhalo. \
                        //Množství nesmí být 0.", num);
                    },
                    Err(&3) => {
                        //println!("[{}] Přidání materiálu selhalo. \
                        //Materiál je již v databázi.", num);
                    },
                    Err(_) => {
                        println!("[{}] Přidání materiálu selhalo. \
                        Neznámá chyba.", num);
                    }
                }
            },
            1 => {
                match evgen {
                    Ok(result) => {
                        println!("[{}] Přidávám produkt \"{}\" složen z {} kusů materiálu \"{}\" \
                        do databáze", num, result.name, result.amount, result.material_id);
                        f1_count +=1;
                    },
                    Err(&1) => {
                        //println!("[{}] Přidání produktu selhalo. \
                        //Název produktu nesmí být prázdný nebo obsahovat netiskuté znaky.", num);
                    }
                    Err(&2) => {
                        //println!("[{}] Přidání produktu selhalo. \
                        //Název materiálu nesmí být prázdný nebo obsahovat netiskuté znaky.", num);
                    }
                    Err(&3) => {
                        //println!("[{}] Přidání produktu selhalo. \
                        //Požadované množství materiálu nesmí být nula", num);
                    }
                    Err(&4) => {
                        println!("[{}] Přidání produktu selhalo. \
                        Materiál neexistuje.", num);
                    }
                    Err(&5) => {
                        //println!("[{}] Přidání produktu selhalo. \
                        //Produkt již existuje.", num);
                    }
                    Err(_) => {
                        println!("[{}] Přidání produktu selhalo. Neznámá chyba.", num);
                    }
                }
            },
            2|3|4|5|6 => {
                match evgen {
                    Ok(result) => {
                        //let tmp = instance.get_material_scarcity(&result.material_id);
                        match result.code {
                            &4 => {
                                println!("[{}] Výroba {} produktů \"{}\" ZAMÍTNUTA. \
                        Materiál \"{}\" není k dispozici; nedostatkovost: {}", num, result.amount, result.name, result.material_id,
                                         get_material_scarcity(instance, &result.material_id));
                                failed_no_supply +=1;
                            },
                            &5 => { println!("[{}] Výroba {} produktů \"{}\" ZAMÍTNUTA. \
                        Materiál \"{}\" nedostatkový: {} > 50.", num, result.amount, result.name, result.material_id,
                                           get_material_scarcity(instance, &result.material_id));
                                failed_scarce +=1;
                            },
                            &_ => println!("[{}] Vyrábím produkt \"{}\" \
                        za cenu {} kusů materiálu \"{}\", nedostatkovost: {}",
                                           num, result.name, result.amount, result.material_id,
                                           get_material_scarcity(instance, &result.material_id)),
                        }
                        f2_count +=1;
                    },
                    Err(&2) => {
                        //println!("[{}] Výroba produktu selhala. \
                        //Nelze objednat 0 kusů.", num);
                    },
                    Err(&3) => {
                        //println!("[{}] Výroba produktu selhala. \
                        //Materiál není v databázi", num);
                    },
                    Err(&4) => {
                        println!("[{}] Výroba produktu selhala. \
                        Materiál neí k dispozici", num); //safeguard for future code changes
                        panic!("Material not available.");
                    },
                    Err(&5) => {
                        println!("[{}] Výroba produktu selhala. \
                        Materiál je vzácný.", num); //safeguard for future code changes
                        panic!("Material scarce.");
                    },
                    Err(&6) => {
                        //println!("[{}] Výroba produktu selhala. \
                        //Databáze produktů je prázdná.", num);
                    },
                    Err(_) => {
                        println!("[{}] Výroba produktu selhala. \
                        Neznámá chyba.", num);
                    }
                }
            },
            7|8|9 => {
                match evgen {
                    Ok(result) => {
                        println!("[{}] Aktualizuji nabídku materiálu \"{}\" na {} k.; \
                        poptávka: {}, nedostatkovost: {}", num, result.name, result.amount,
                                 get_material_demand(instance, &result.name),
                                 tst_get_material(instance, &result.name).calculate_scarcity()
                                 );
                        f3_count +=1;
                    },
                    Err(&1) => {
                        println!("[{}] Aktualizace nabídky materiálu selhala. \
                        Databáze materiálů je prázdná.", num);
                    },
                    Err(&2) => {
                        println!("[{}] Aktualizace nabídky materiálu selhala. \
                        Proces selhal.", num);
                    },
                    Err(_) => {
                        println!("[{}] Aktualizace nabídky materiálu selhala. \
                        Neznámá chyba.", num);
                    }
                }
            },
            _ => { panic!("Out of range.")}
        }

        num += 1;
        if millis != 0 { thread::sleep(time); }
    }
    println!("\nProgram skončil v cyklu {}.\n\
    Vykonané funkce      | Přidej materiál: {}, Přidej produkt: {}, Objednej produkt: {}, Aktualizuj nabídku: {}",
             num, f0_count, f1_count, f2_count, f3_count);
    println!("Neúspěšné objednávky | nedostatečná nabídka: {}, vzácnost: {}", failed_no_supply, failed_scarce);
}