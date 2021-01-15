health_sources = files(
  'core/application.vala',
  'core/database.vala',
  'core/settings.vala',
  'core/units.vala',
  'core/util.vala',
  'model/activity.vala',
  'model/activity_model.vala',
  'model/graph_model.vala',
  'model/steps_graph_model.vala',
  'model/weight_graph_model.vala',
  'sync/google_fit_oauth2_proxy.vala',
  'sync/oauth2_proxy.vala',
  'views/activity_view.vala',
  'views/graph_view.vala',
  'views/steps_view.vala',
  'views/view.vala',
  'views/weight_view.vala',
  'widgets/activity_row.vala',
  'widgets/activity_type_selector.vala',
  'widgets/bmi_level_bar.vala',
  'widgets/date_editor.vala',
  'widgets/distance_action_row.vala',
  'widgets/sync_list_box.vala',
  'windows/activity_add_dialog.vala',
  'windows/preferences_window.vala',
  'windows/setup_window.vala',
  'windows/weight_add_dialog.vala',
  'windows/window.vala',
)

cc = meson.get_compiler('c')

health_deps = [
  dependency('gio-2.0', version: '>= 2.50'),
  dependency('gtk4', version: '>=3.99.6'),
  dependency('libadwaita-1'),
  dependency('gee-0.8'),
  dependency('libsoup-2.4'),
  dependency('rest-0.7'),
  dependency('libsecret-1'),
  dependency('json-glib-1.0'),
  dependency('pangocairo'),
  dependency('tracker-sparql-3.0'),
  cc.find_library('m', required: false),
]

# Add our config VAPI
add_project_arguments(
  [
    '--vapidir', meson.current_source_dir() / 'core',
    '--pkg', 'config',
  ],
  language: 'vala'
)

# Configuration
conf = configuration_data()
conf.set_quoted('APPLICATION_ID', application_id)
conf.set_quoted('VERSION', health_version)
conf.set_quoted('DATADIR', health_data_dir)

configure_file(
  output: 'config.h',
  configuration: conf,
)

# Include config.h
add_project_arguments(
  '-I@0@'.format(meson.current_build_dir()),
  language: 'c'
)

c_args = [
  '-DVERSION="@0@"'.format(meson.project_version()),
  '-DGETTEXT_PACKAGE="@0@"'.format(meson.project_name()),
  '-DGNOMELOCALEDIR="@0@"'.format(join_paths(get_option('prefix'), get_option('localedir'))),
  '-DG_LOG_DOMAIN="Health"',
]


lib = static_library(
  application_id,
  [health_sources, health_generated_sources],
  vala_args: '--target-glib=2.50',
  dependencies: health_deps,
  c_args: c_args,
)

lib_dep = declare_dependency(
  dependencies: health_deps,
  link_with: lib,
)


executable(
  application_id,
  ['core/main.vala', health_generated_sources],
  vala_args: '--target-glib=2.50',
  dependencies: lib_dep,
  c_args: c_args,
  install: true,
)
