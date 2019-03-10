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

extern crate resource_mng;
extern crate rand;

pub mod event_generator;

#[cfg(target_family = "windows")]
use std::io::Read;

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
        if cfg!(windows) { millis = 1500; } else { millis = 0 }
    }
    let mut rng = rand::thread_rng();
    let mut instance = init();
    instance.verbose = VERBOSITY_INNER;
    let verbose = instance.verbose;
    let mut instance = &mut instance;
    let mut num: usize = 0;
    let mut f0_count: usize = 0;
    let mut f1_count: usize = 0;
    let mut f2_count: usize = 0;
    let mut f3_count: usize = 0;
    let mut failed_scarce: usize = 0;
    let mut failed_no_supply: usize = 0;
    let sleep = time::Duration::from_millis(millis);
    let mut fn_num;
    let max_values: usize = 512;
    let mut evgen;

    if cfg!(feature = "cz") {
        println!("Spouštím simulaci na {} cyklů s pauzou {} ms.\n\
        Generuji úvodní položky databáze, prosím čekejte...", cycles, millis);
    } else {
        println!("Starting simulation for {} cycles with {} ms pause.\n\
        Generating initial database entries, please wait...", cycles, millis);
    }
    event_generator::init(instance, &mut rng, max_values, cycles);

    while num < cycles || cycles == 0 {
        fn_num = rng.gen::<u8>() % 10;
        evgen = event_generator::run(&mut instance, fn_num, &mut rng, max_values);
        match fn_num {
            //add material
            0 => {
                match evgen {
                    Ok(result) => {
                        if cfg!(feature = "cz") {
                            println!("[{}] Přidávám materiál #{} do databáze; množství: {}",
                                     num, result.primary_id, result.amount);
                        } else {
                            println!("[{}] Adding material #{} to the database, supply: {}",
                                     num, result.primary_id, result.amount);
                        }
                        f0_count += 1;
                    }
                    Err(1) => {
                        if cfg!(feature = "cz") {
                            println!("[{}] Přidání materiálu selhalo. \
                            Název nesmí být prázdný nebo obsahovat netiskuté znaky.", num);
                        } else {
                            println!("[{}] Adding material failed. \
                            Name cannot be empty or contain only white spaces.", num);
                        }
                    }
                    Err(2) => {
                        if verbose >= 3 {
                            if cfg!(feature = "cz") {
                                println!("[{}] Přidání materiálu selhalo. \
                                Množství nesmí být 0.", num);
                            } else {
                                println!("[{}] Adding material failed. \
                                Supply cannot be zero.", num);
                            }
                        }
                    }
                    Err(3) => {
                        if verbose >= 3 {
                            if cfg!(feature = "cz") {
                                println!("[{}] Přidání materiálu selhalo. \
                                Materiál je již v databázi.", num);
                            } else {
                                println!("[{}] Adding material failed. \
                                Material already in database.", num);
                            }
                        }
                    }
                    Err(_) => {
                        if cfg!(feature = "cz") {
                            println!("[{}] Přidání materiálu selhalo. \
                            Neznámá chyba.", num);
                        } else {
                            println!("[{}] Adding material failed. \
                            Unknown error.", num);
                        }
                    }
                }
            }
            // add product
            1 => {
                match evgen {
                    Ok(result) => {
                        if cfg!(feature = "cz") {
                            println!("[{}] Přidávám produkt #{} složen z {} kusů materiálu #{} \
                        do databáze", num, result.primary_id, result.amount, result.secondary_id);
                        } else {
                            println!("[{}] Adding product #{} composed of {}x material #{} \
                        to the database", num, result.primary_id, result.amount, result.secondary_id);
                        }
                        f1_count += 1;
                    }
                    Err(1) => {
                        if verbose >= 3 {
                            if cfg!(feature = "cz") {
                                println!("[{}] Přidání produktu selhalo. \
                                Název produktu nesmí být prázdný nebo obsahovat netiskuté znaky.", num);
                            } else {
                                println!("[{}] Adding product failed. \
                                Product name cannot be empty or contain only white spaces.", num);
                            }
                        }
                    }
                    Err(2) => {
                        if verbose >= 3 {
                            if cfg!(feature = "cz") {
                                println!("[{}] Přidání produktu selhalo. \
                        Název materiálu nesmí být prázdný nebo obsahovat netiskuté znaky.", num);
                            } else {
                                println!("[{}] Adding product failed. \
                        Material name cannot be empty or contain only white spaces.", num);
                            }
                        }
                    }
                    Err(3) => {
                        if verbose >= 3 {
                            if cfg!(feature = "cz") {
                                println!("[{}] Přidání produktu selhalo. \
                        Požadované množství materiálu nesmí být nula a méně", num);
                            } else {
                                println!("[{}] Adding product failed. \
                        Material amount required must not be zero and less.", num);
                            }
                        }
                    }
                    Err(4) => {
                        if cfg!(feature = "cz") {
                            println!("[{}] Přidání produktu selhalo. \
                        Materiál neexistuje.", num);
                        } else {
                            println!("[{}] Adding product failed. \
                        Material does not exist.", num);
                        }
                    }
                    Err(5) => {
                        if verbose >= 3 {
                            if cfg!(feature = "cz") {
                                println!("[{}] Přidání produktu selhalo. \
                        Produkt již existuje.", num);
                            } else {
                                println!("[{}] Adding product failed. \
                        Product already exists.", num);
                            }
                        }
                    }
                    Err(_) => {
                        if cfg!(feature = "cz") {
                            println!("[{}] Přidání produktu selhalo. Neznámá chyba.", num);
                        } else { println!("[{}] Adding product failed. Unknown error.", num); }
                    }
                }
            }
            // order product
            2 | 3 | 4 | 5 => {
                match evgen {
                    Ok(result) => {
                        match *result.code {
                            4 => {
                                if cfg!(feature = "cz")
                                {
                                    println!("[{}] Výroba {} produktů #{} ZAMÍTNUTA. \
                        Materiál #{} není k dispozici; nedostatkovost: {:.2}", num, result.amount, result.primary_id, result.secondary_id,
                                             get_material_scarcity(instance, &result.secondary_id));
                                } else {
                                    println!("[{}] Manufacturing of {}x product #{} DENIED. \
                        Material #{} not available; scarcity: {:.2}", num, result.amount, result.primary_id, result.secondary_id,
                                             get_material_scarcity(instance, &result.secondary_id));
                                }
                                failed_no_supply += 1;
                            }
                            5 => {
                                if cfg!(feature = "cz") {
                                    println!("[{}] Výroba {} produktů #{} ZAMÍTNUTA. \
                        Materiál #{} nedostatkový: {:.2} > 50.", num, result.amount, result.primary_id, result.secondary_id,
                                             get_material_scarcity(instance, &result.secondary_id));
                                } else {
                                    println!("[{}] Manufacturing of {}x product #{} DENIED. \
                        Material #{} scarce: {:.2} > 50.", num, result.amount, result.primary_id, result.secondary_id,
                                             get_material_scarcity(instance, &result.secondary_id));
                                }
                                failed_scarce += 1;
                            }
                            _ => {
                                if cfg!(feature = "cz") {
                                    println!("[{}] Objednávám produkt #{} \
                        za cenu {} kusů materiálu #{}, nedostatkovost: {:.2}",
                                             num, result.primary_id, result.amount, result.secondary_id,
                                             get_material_scarcity(instance, &result.secondary_id))
                                } else {
                                    println!("[{}] Ordering product #{} \
                        at the cost of {}x material #{}; scarcity: {:.2}",
                                             num, result.primary_id, result.amount, result.secondary_id,
                                             get_material_scarcity(instance, &result.secondary_id))
                                }
                            }
                        }
                        f2_count += 1;
                    }
                    Err(2) => {
                        if verbose >= 3 {
                            if cfg!(feature = "cz") {
                                println!("[{}] Výroba produktu selhala. \
                                Nelze objednat 0 kusů.", num);
                            } else {
                                println!("[{}] Manufacturing product failed. \
                        Cannot order 0 products.", num);
                            }
                        }
                    }
                    Err(3) => {
                        if verbose >= 3 {
                            if cfg!(feature = "cz") {
                                println!("[{}] Výroba produktu selhala. \
                        Materiál není v databázi", num);
                            } else {
                                println!("[{}] Manufacturing product failed. \
                        No such material in database.", num);
                            }
                        }
                    }
                    Err(4) => {
                        if cfg!(feature = "cz") {
                            println!("[{}] Výroba produktu selhala. \
                        Materiál neí k dispozici", num);
                        } else {
                            println!("[{}] Manufacturing product failed. \
                        Material not available.", num);
                        } //safeguard for future code changes
                        panic!("Material not available.");
                    }
                    Err(5) => {
                        if cfg!(feature = "cz") {
                            println!("[{}] Výroba produktu selhala. \
                        Materiál je vzácný.", num);
                        } else {
                            println!("[{}] Manufacturing product failed. \
                        Material scarce.", num); //safeguard for future code changes
                        }
                        panic!("Material scarce.");
                    }
                    Err(6) => {
                        if verbose >= 3 {
                            if cfg!(feature = "cz") {
                                println!("[{}] Výroba produktu selhala. \
                         Databáze produktů je prázdná.", num);
                            } else {
                                println!("[{}] Manufacturing product failed. \
                        Product database is empty.", num);
                            }
                        }
                    }
                    Err(_) => {
                        if cfg!(feature = "cz") {
                            println!("[{}] Výroba produktu selhala. \
                        Neznámá chyba.", num);
                        } else {
                            println!("[{}] Manufacturing product failed. \
                        Unknown error.", num);
                        }
                    }
                }
            }
            // add product variant
            6 | 7 => {
                match evgen {
                    Ok(result) => {
                        if cfg!(feature = "cz") {
                            println!("[{}] Přidávám novou variantu produktu #{} \
                        z materiálu #{}, komplexita {:.2}.", num, result.primary_id, result.secondary_id, result.work_complexity);
                        } else {
                            println!("[{}] Adding new variant to product #{} \
                        consisting of material #{}, complexity {:.2}.", num, result.primary_id, result.secondary_id, result.work_complexity);
                        }
                    }
                    Err(_) => {
                        if cfg!(feature = "cz") {
                            println!("[{}] Přidání nové varianty selhalo. Produkt nebo materiál neexistuje.", num);
                        } else { println!("[{}] Adding new variant failed. No such product or material.", num); }
                    }
                }
            }
            // update supply
            8 | 9 => {
                match evgen {
                    Ok(result) => {
                        if cfg!(feature = "cz") {
                            println!("[{}] Aktualizuji nabídku materiálu #{} na {} ks; \
                        poptávka: {}, nedostatkovost: {:.2}", num, result.primary_id, result.amount,
                                     get_material_demand(instance, &result.primary_id),
                                     instance.get_material(result.primary_id).get_scarcity()
                            );
                        } else {
                            println!("[{}] Updating supply of material #{} to {}; \
                        demand: {}, scarcity: {:.2}", num, result.primary_id, result.amount,
                                     get_material_demand(instance, &result.primary_id),
                                     instance.get_material(result.primary_id).get_scarcity()
                            );
                        }
                        f3_count += 1;
                    }
                    Err(1) => {
                        if cfg!(feature = "cz") {
                            println!("[{}] Aktualizace nabídky materiálu selhala. \
                        Databáze materiálů je prázdná.", num);
                        } else {
                            println!("[{}] Updating supply of material failed. \
                        No materials in database.", num);
                        }
                    }
                    Err(2) => {
                        if cfg!(feature = "cz") {
                            println!("[{}] Aktualizace nabídky materiálu selhala. \
                        Proces selhal.", num);
                        } else {
                            println!("[{}] Updating supply of material failed. \
                        Supply update failed.", num);
                        }
                    }
                    Err(_) => {
                        if cfg!(feature = "cz") {
                            println!("[{}] Aktualizace nabídky materiálu selhala. \
                        Neznámá chyba.", num);
                        } else {
                            println!("[{}] Updating supply of material failed. \
                        Unknown error.", num);
                        }
                    }
                }
            }
            _ => { panic!("Out of range.") }
        }

        num += 1;
        if millis != 0 { thread::sleep(sleep); }
    }
    if cfg!(feature = "cz")
    {
        eprintln!("\nProgram skončil v cyklu {}.\n\
    Vykonané funkce      | Přidej materiál: {}, Přidej produkt: {}, Objednej produkt: {}, Aktualizuj nabídku: {}",
                  num, f0_count, f1_count, f2_count, f3_count);
        eprintln!("Neúspěšné objednávky | nedostatečná nabídka: {}, vzácnost: {}", failed_no_supply, failed_scarce);
    } else {
        eprintln!("\nProgram ends at cycle {}.\n\
    Functions passed | Add material: {}, Add product: {}, Order product: {}, Update supply: {}",
                  num, f0_count, f1_count, f2_count, f3_count);
        eprintln!("Failed orders    | no supply: {}, scarce: {}", failed_no_supply, failed_scarce);
    }
    #[cfg(target_family = "windows")]
        std::io::stdin().read(&mut [0u8]).unwrap();
}