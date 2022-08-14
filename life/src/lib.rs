mod main;
use main::main as lib_main;
use mobile_entry_point::mobile_entry_point;

#[mobile_entry_point]
fn mobile_main() {
    lib_main()
}
