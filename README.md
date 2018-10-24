This project is an attempt to create a library providing essential functions present in a Natural Law / Resource Based Economy. To test it out I made a terminal based random event generator so together it is basically an economy simulator. In this early stage it supports adding materials, products and evaluates orders based on material scarcity. The library is **C compatible**. There is an example in the src folder.

The goal is to have complete RBE resource management and production simulated. At the moment the library can do only 4 things. Add new material and depending product to the database, place order for a product which is material-scarcity and priority assessed before authorized and update material supply like when you get new reading from a sensor. In the future it will calculate what material to use to build particular product most efficiently (assembly complexity) and from most abundant materials taking into account user desires. It is about the process from the moment when materials are available to the factory to the moment when products are ready to be shipped somewhere else, perhaps the distribution centers. I have no definite idea in my mind and will expand it as new ideas and problems arrive.

**The program takes two command line parameters. First is number of cycles. Second is number of milliseconds after every cycle.**

To compile yourself you will need to install Rust (including Cargo) and run "cargo build --release" command.

If you like the work please consider sending a [donation](https://www.paypal.com/cgi-bin/webscr?cmd=_donations&business=mauserm@seznam.cz&item_name=Resource%20management&item_number=Development).
