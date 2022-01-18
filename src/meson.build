conf = configuration_data()
conf.set_quoted('VERSION', meson.project_version())
conf.set_quoted('localedir', get_option('prefix') / get_option('localedir'))
conf.set_quoted('pkgdatadir', health_data_dir)
conf.set_quoted('application_id', application_id)
conf.set_quoted('log_domain', application_id)
conf.set_quoted('gettext_package', gettext_package)
conf.set_quoted('autostart_desktop_file_path', get_option('prefix') / get_option('datadir') / 'applications' / 'dev.Cogitri.Health.Autostart.desktop')

configure_file(
    input: 'config.rs.in',
    output: 'config.rs',
    configuration: conf
)

# Copy the config.rs output to the source directory.
run_command(
  'cp',
  meson.project_build_root() / 'src' / 'config.rs',
  meson.project_source_root() / 'src' / 'config.rs',
  check: true
)

sources = files(
  'config.rs',
  'core/application.rs',
  'core/database.rs',
  'core/date.rs',
  'core/i18n.rs',
  'core/macros.rs',
  'core/mod.rs',
  'core/ref_iter.rs',
  'core/settings.rs',
  'core/unit_kind.rs',
  'core/unit_system.rs',
  'core/utils.rs',
  'lib.rs',
  'main.rs',
  'model/activity_data_points.rs',
  'model/activity_info.rs',
  'model/activity.rs',
  'model/activity_type_row_data.rs',
  'model/activity_type.rs',
  'model/fn_boxed.rs',
  'model/model_notification.rs',
  'model/mod.rs',
  'model/steps.rs',
  'model/unitsize.rs',
  'model/weight_change.rs',
  'model/weight.rs',
  'plugins/activities/data_provider_mocked.rs',
  'plugins/activities/data_provider.rs',
  'plugins/activities/details.rs',
  'plugins/activities/mod.rs',
  'plugins/activities/plugin.rs',
  'plugins/activities/summary.rs',
  'plugins/calories/data_provider_mocked.rs',
  'plugins/calories/data_provider.rs',
  'plugins/calories/details.rs',
  'plugins/calories/mod.rs',
  'plugins/calories/plugin.rs',
  'plugins/calories/summary.rs',
  'plugins/details.rs',
  'plugins/mod.rs',
  'plugins/overview.rs',
  'plugins/plugin_list.rs',
  'plugins/plugin_name.rs',
  'plugins/plugin_object.rs',
  'plugins/plugin.rs',
  'plugins/registrar.rs',
  'plugins/steps/data_provider_mocked.rs',
  'plugins/steps/data_provider.rs',
  'plugins/steps/details.rs',
  'plugins/steps/mod.rs',
  'plugins/steps/plugin.rs',
  'plugins/steps/summary.rs',
  'plugins/summary.rs',
  'plugins/weight/data_provider_mocked.rs',
  'plugins/weight/data_provider.rs',
  'plugins/weight/details.rs',
  'plugins/weight/mod.rs',
  'plugins/weight/plugin.rs',
  'plugins/weight/summary.rs',
  'sync/csv.rs',
  'sync/database_receiver.rs',
  'sync/google_fit.rs',
  'sync/mod.rs',
  'sync/serialize.rs',
  'sync/sync_provider.rs',
  'views/bar_graph_view.rs',
  'views/graph_view.rs',
  'views/mod.rs',
  'views/view_add_activity.rs',
  'views/view_add.rs',
  'views/view_add_weight.rs',
  'views/view_home_page.rs',
  'widgets/activity_row.rs',
  'widgets/activity_type_row.rs',
  'widgets/activity_type_selector.rs',
  'widgets/arrows.rs',
  'widgets/bmi_level_bar.rs',
  'widgets/card.rs',
  'widgets/circular_progress_bar.rs',
  'widgets/color_circle.rs',
  'widgets/date_selector.rs',
  'widgets/distance_action_row.rs',
  'widgets/legend_row.rs',
  'widgets/mod.rs',
  'widgets/password_entry.rs',
  'widgets/sync_list_box.rs',
  'widgets/tab_button.rs',
  'widgets/unit_spinbutton.rs',
  'windows/data_add_dialog.rs',
  'windows/export_dialog.rs',
  'windows/import_dialog.rs',
  'windows/import_export_dialog_base.rs',
  'windows/mod.rs',
  'windows/preferences_window.rs',
  'windows/setup_window.rs',
  'windows/window.rs',
)

cargo_release = custom_target(
  'cargo-build',
  build_by_default: true,
  input: sources,
  output: [application_id],
  console: true,
  install: true,
  install_dir: get_option('bindir'),
  command: [
    find_program(meson.project_source_root() / 'build-aux' / 'cargo.sh'),
    meson.current_build_dir() / '..',
    meson.project_source_root(),
    '@OUTPUT0@',
    get_option('buildtype'),
    'health',
  ]
)

test(
  'cargo-tests',
  find_program(meson.project_source_root() / 'build-aux' / 'cargo-test.sh'),
  args: [
    meson.current_build_dir() / '..',
    meson.project_source_root(),
    get_option('buildtype'),
  ],
  timeout: 3000, # might take a bit longer due to cargo...
)
