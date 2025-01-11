use emacs_mac_input_source::tis;

fn main() {
    let filter_props = tis::TISInputSourceProperties {
        is_select_capable: Some(true),
        ..Default::default()
    };
    let input_sources = tis::TISInputSource::new_list(&filter_props, false);
    for input_source in &input_sources {
        let props = input_source.get_properties();
        println!("{:?}", props);
    }
    // println!("Hello, world!");
    // let input_source = tis::TISInputSource::current_keyboard();
    // let props = input_source.get_properties();
    // println!("{:?}", props);
}
