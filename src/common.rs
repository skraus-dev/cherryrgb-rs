pub fn get_u16_from_string(pid: Option<String>) -> Option<u16> {
    let cpid = pid.clone();
    if let Some(stripped) = cpid?.strip_prefix("0x") {
        let val = u16::from_str_radix(stripped, 16).ok()?;
        return Some(val);
    }
    let val = pid?.as_str().parse::<u16>().ok()?;
    Some(val)
}
