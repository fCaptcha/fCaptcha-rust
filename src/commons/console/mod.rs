use ansi_hex_color;
use crate::ARGUMENTS;

pub enum SolveType {
    CUSTOMER,
    INTERNAL
}

pub async fn solved(solve_type: SolveType, token: Option<&str>, variant: Option<&str>, waves: Option<&i32>, solved: Option<&bool>) -> Option<()> {
    if !ARGUMENTS.print_bad_captcha_results && !*solved? {
        return None;
    }
    let solve_type_str = match solve_type {
        SolveType::CUSTOMER => {
            "CUSTOMER SOLVE"
        }
        SolveType::INTERNAL => {
            "INTERNAL SOLVE"
        }
    };
    let sol = *solved?;
    let col_st = ansi_hex_color::colored("#00FF7F", "", solve_type_str);
    let col_token = ansi_hex_color::colored("#7FFFD4", "", token?.split("|").next()?);
    let col_variant = ansi_hex_color::colored("#7FFFD4", "", variant?);
    let col_waves  = ansi_hex_color::colored("#7FFFD4", "", &*waves?.to_string());
    let col_solved  = ansi_hex_color::colored("#7FFFD4", "", &*sol.to_string());
    let col_sep = ansi_hex_color::colored("#91A3B0", "", "|");
    let col_info  = ansi_hex_color::colored("#91A3B0", "", "");
    println!("{col_st}{col_info} T: {col_token} {col_sep}{col_info} W: {col_waves} {col_sep}{col_info} V: {col_variant} {col_sep}{col_info} S: {col_solved}");
    Some(())
}

pub fn created_account(credentials: Option<&str>) -> Option<()> {
    let colored_0 = ansi_hex_color::colored(
        "#00FF7F", "", "CREATED");

    let colored_1 = ansi_hex_color::colored(
        "#7FFFD4", "", credentials?);

    println!("{} {}", colored_0, colored_1);
    Some(())
}

// pub fn sent_proof(value: Option<&Value>) -> Option<()> {
//     let colored_0 = ansi_hex_color::colored(
//         "#00FF7F", "", "SENT HSW");
//     let mut data = value?.to_string();
//     let _ = data.split_off(33);
//     let colored_1 = ansi_hex_color::colored(
//         "#7FFFD4", "", &*data.replace("\"", ""));
//     println!("{} {}", colored_0, colored_1);
//     Some(())
// }