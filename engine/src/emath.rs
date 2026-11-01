pub struct EMath {
    // stuff
}

impl EMath {
    pub fn new() -> Self {
        EMath {}
    }

    pub fn normalize(&self, input: f64, deci: u32) -> f64 {
        let factor = 10f64.powi(deci as i32);
        let out: f64 = (input * factor).round() / factor;
        return out;
    }

    pub fn to_str<T: ToString>(&self, value: T) -> String {
        let out: String = value.to_string();
        return out;
    }
}
