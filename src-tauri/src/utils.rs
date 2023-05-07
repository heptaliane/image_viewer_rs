use super::image::try_get_source_image;
use super::state::ViewerState;

pub fn get_next_image(state: &mut ViewerState, moves: i32) -> Result<String, String> {
    for _ in 0..moves {
        if let Err(err) = state.next_cursor() {
            return Err(err);
        }
    }

    loop {
        match state.get() {
            Ok(path) => match try_get_source_image(&path) {
                Ok(img) => {
                    log::debug!("Current image: {:?}", path);
                    return Ok(img);
                }
                Err(err) => log::info!("{:?}", err),
            },
            Err(err) => log::info!("{:?}", err),
        }

        if let Err(err) = state.next_cursor() {
            return Err(err);
        }
    }
}

pub fn get_prev_image(state: &mut ViewerState, moves: i32) -> Result<String, String> {
    for _ in moves..0 {
        if let Err(err) = state.prev_cursor() {
            return Err(err);
        }
    }

    loop {
        match state.get() {
            Ok(path) => match try_get_source_image(&path) {
                Ok(img) => {
                    log::debug!("Current image: {:?}", path);
                    return Ok(img);
                }
                Err(err) => log::info!("{:?}", err),
            },
            Err(err) => log::info!("{:?}", err),
        }

        if let Err(err) = state.prev_cursor() {
            return Err(err);
        }
    }
}
