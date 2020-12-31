health_sources = files(
  'activity.vala',
  'activity_row.vala',
  'activity_view.vala',
  'activity_add_dialog.vala',
  'application.vala',
  'bmi_level_bar.vala',
  'database.vala',
  'date_editor.vala',
  'graph_view.vala',
  'oauth2_proxy.vala',
  'preferences_window.vala',
  'settings.vala',
  'setup_window.vala',
  'steps.vala',
  'sync_view.vala',
  'units.vala',
  'util.vala',
  'views.vala',
  'weight.vala',
  'weight_add_dialog.vala',
  'window.vala',
)

cc = meson.get_compiler('c')

health_deps = [
  dependency('gio-2.0', version: '>= 2.50'),
  dependency('gtk4'),
  dependency('libhandy-4'),
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
    '--vapidir', join_paths(meson.current_source_dir()),
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
  ['main.vala', health_generated_sources],
  vala_args: '--target-glib=2.50',
  dependencies: lib_dep,
  c_args: c_args,
  install: true,
)
