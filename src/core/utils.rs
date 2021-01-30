use gtk::EditableExt;

pub fn get_spinbutton_value<T>(spin_button: &gtk::SpinButton) -> T
where
    T: std::str::FromStr + Default,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    spin_button
        .get_text()
        .unwrap()
        .as_str()
        .parse::<T>()
        .unwrap_or(T::default())
}
