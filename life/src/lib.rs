mod main;
use main::main as main_main;
use mobile_entry_point::mobile_entry_point;

#[mobile_entry_point]
fn mobile_main() {
    main_main()
}
