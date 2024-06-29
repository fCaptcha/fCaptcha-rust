pub fn get_threshold(variant: &str) -> u32 {
    let mut ret = 16;
    match variant {
        "3d_rollball_objects" => {
            ret = 64;
        }
        "coordinatesmatch" => {
            ret = 36;
        }
        "orbit_match_game" => {
            ret = 72;
        }
        "hopscotch_highsec" => {
            ret = 109;
        }
        "rockgroup" => {
            ret = 88;
        }
        "numericalmatch" => {
            ret = 60;
        }
        _ => {}
    }
    ret
}