use emacs_mac_input_source::tis;

fn do_list() {
    let filter_props = tis::TISInputSourceProperties {
        is_select_capable: Some(true),
        ..Default::default()
    };
    let input_sources = tis::TISInputSource::new_list(&filter_props, false);
    for input_source in &input_sources {
        let props = input_source.get_properties();
        if let tis::TISInputSourceProperties{
            id: Some(ref id),
            localized_name: Some(ref name),
            is_selected: Some(selected),
            ..
        } = props {
            println!("{} {} ({})", if selected { "*" } else { " " }, id, name);
            println!("{:?}\n", props);
        }
    }
}

fn do_select(id: &str) -> anyhow::Result<()> {
    let filter_props = tis::TISInputSourceProperties {
        is_select_capable: Some(true),
        id: Some(id.into()),
        ..Default::default()
    };
    let input_sources = tis::TISInputSource::new_list(&filter_props, false);
    if input_sources.len() != 1 {
        anyhow::bail!("Cannot find input source {}", id);
    }
    input_sources[0].select()?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() == 1 {
        do_list();
        Ok(())
    } else {
        do_select(&args[1])
    }
}
