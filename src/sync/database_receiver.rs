use crate::{
    core::{i18n_f, HealthDatabase},
    model::{Steps, Weight},
};
use gtk_macros::spawn;

pub enum DatabaseValue {
    Steps(Vec<Steps>),
    Weights(Vec<Weight>),
}

pub fn new_db_receiver(db: HealthDatabase) -> glib::Sender<DatabaseValue> {
    let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    receiver.attach(None, move |value| {
        let db = db.clone();
        match value {
            DatabaseValue::Steps(s) => {
                spawn!(async move {
                    if let Err(e) = db.import_steps(&s).await {
                        glib::g_warning!(
                            crate::config::LOG_DOMAIN,
                            "{}",
                            i18n_f(
                                "Couldn't synchronise steps due to error {}",
                                &[&e.to_string()]
                            )
                        );
                    }
                });
            }
            DatabaseValue::Weights(w) => {
                spawn!(async move {
                    if let Err(e) = db.import_weights(&w).await {
                        glib::g_warning!(
                            crate::config::LOG_DOMAIN,
                            "{}",
                            i18n_f(
                                "Couldn't synchronise weight measurements due to error {}",
                                &[&e.to_string()]
                            )
                        );
                    }
                });
            }
        }

        glib::Continue(true)
    });

    sender
}
