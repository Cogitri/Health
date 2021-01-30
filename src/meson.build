conf = configuration_data()
conf.set_quoted('VERSION', meson.project_version())
conf.set_quoted('localedir', join_paths(get_option('prefix'), get_option('localedir')))
conf.set_quoted('pkgdatadir', health_data_dir)
conf.set_quoted('application_id', application_id)
conf.set_quoted('log_domain', application_id)

configure_file(
    input: 'config.rs.in',
    output: 'config.rs',
    configuration: conf
)

# Copy the config.rs output to the source directory.
run_command(
  'cp',
  join_paths(meson.build_root(), 'src', 'config.rs'),
  join_paths(meson.source_root(), 'src', 'config.rs'),
  check: true
)

sources = files(
  'main.rs',
  'config.rs',
  'core/application.rs',
  'core/mod.rs',
  'core/i18n.rs',
  'core/macros.rs',
  'core/utils.rs',
  'core/settings.rs',
  'core/database.rs',
  'model/mod.rs',
  'model/activity.rs',
  'model/activity_type_row_data.rs',
  'model/activity_type.rs',
  'model/activity_info.rs',
  'model/activity_data_points.rs',
  'model/graph_model_steps.rs',
  'model/graph_model_weight.rs',
  'model/model_activity.rs',
  'windows/mod.rs',
  'windows/setup_window.rs',
  'windows/preferences_window.rs',
  'windows/activity_add_dialog.rs',
  'windows/weight_add_dialog.rs',
  'windows/window.rs',
  'views/graph_view.rs',
  'views/mod.rs',
  'views/view.rs',
  'views/view_steps.rs',
  'views/view_weight.rs',
  'views/view_activity.rs',
  'widgets/activity_row.rs',
  'widgets/activity_type_row.rs',
  'widgets/mod.rs',
  'widgets/activity_type_selector.rs',
  'widgets/bmi_level_bar.rs',
  'widgets/date_selector.rs',
  'widgets/distance_action_row.rs',
  'widgets/sync_list_box.rs',
)

cargo_script = find_program(join_paths(meson.source_root(), 'build-aux/cargo.sh'))
cargo_release = custom_target(
  'cargo-build',
  build_by_default: true,
  input: sources,
  output: applcation_id,
  console: true,
  install: true,
  install_dir: get_option('bindir'),
  command: [
    cargo_script,
    meson.build_root(),
    meson.source_root(),
    '@OUTPUT@',
    get_option('buildtype'),
    'health',
  ]
)
