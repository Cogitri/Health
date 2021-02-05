use gtk::EditableExt;
#[cfg(test)]
use std::future::Future;

pub fn get_spinbutton_value<T>(spin_button: &gtk::SpinButton) -> T
where
    T: std::str::FromStr + Default,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    spin_button
        .get_text()
        .as_str()
        .parse::<T>()
        .unwrap_or_default()
}

#[cfg(test)]
pub fn run_async_test_fn<T: 'static, F: 'static>(future: F) -> T
where
    F: Future<Output = T>,
{
    let context = glib::MainContext::new();
    let ml = glib::MainLoop::new(Some(&context), false);
    let (sender, receiver) = std::sync::mpsc::channel();

    context.push_thread_default();
    let m = ml.clone();
    context.spawn_local(async move {
        sender.send(future.await).unwrap();
        m.quit();
    });

    ml.run();

    receiver.recv().unwrap()
}
