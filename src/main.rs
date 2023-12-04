use breadx::{
    connection::BufConnection,
    display::{BasicDisplay, DisplayConnection},
    prelude::{Display, DisplayBase, DisplayFunctionsExt},
    protocol::{
        xproto::{AtomEnum, ChangeWindowAttributesAux, EventMask},
        Event::PropertyNotify,
    },
    NameConnection,
};

use std::error::Error;

/**
 * @see https://github.com/dimusic/active-win-pos-rs/blob/c0bec6433f79d3a8986c9d73fbe318a13562c641/src/linux/platform_api.rs#L98
 */
fn get_active_window(
    connection: &mut BasicDisplay<BufConnection<NameConnection>>,
    root: u32,
    atom: u32,
) -> Result<u32, ()> {
    let response = connection
        .get_property_immediate(false, root, atom, u8::from(AtomEnum::WINDOW), 0, 1)
        .unwrap();

    if response.value32().is_none() {
        return Err(());
    }

    Ok(response.to_owned().value32().unwrap().next().unwrap())
}

fn main() -> Result<(), Box<dyn Error>> {
    /*
     * @see https://docs.rs/breadx/3.1.0/breadx/
     */
    let mut connection = DisplayConnection::connect(None).expect("should connect to x11 server");
    let root = connection.default_screen().root;
    let net_active_window = connection
        .intern_atom_immediate(true, "_NET_ACTIVE_WINDOW")
        .unwrap()
        .atom;

    /*
     * @see https://gist.github.com/ssokolow/e7c9aae63fb7973e4d64cff969a78ae8
     */
    if let Ok(active_window) = get_active_window(&mut connection, root, net_active_window) {
        let window_id = &active_window;

        connection.change_window_attributes_checked(
            root,
            ChangeWindowAttributesAux::default().event_mask(EventMask::PROPERTY_CHANGE),
        )?;

        // primary event loop
        loop {
            let event = connection.wait_for_event();

            match event {
                // match on the Event struct in here
                Ok(PropertyNotify(e)) => {
                    if e.atom == net_active_window {
                        if let Ok(active_window) =
                            get_active_window(&mut connection, root, net_active_window)
                        {
                            if &active_window != window_id {
                                match connection.unmap_window_checked(*window_id) {
                                    Ok(_) => (),
                                    Err(err) => {
                                        eprintln!("Error unmapping window: {:?}", err);
                                        break;
                                    }
                                }
                            }
                        } else {
                            eprintln!("Error getting active window");
                            break;
                        }
                    }
                }
                Err(_) => {
                    eprintln!("X11 server has crashed, exiting program.");
                    std::process::exit(1);
                }
                Ok(_) => todo!(),
            }
        }
    } else {
        eprintln!("Error getting initial active window");
    }

    Ok(())
}
