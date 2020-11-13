health_sources = files(
  'add_dialog.vala',
  'application.vala',
  'database.vala',
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
  'window.vala',
)

cc = meson.get_compiler('c')

health_deps = [
  dependency('gio-2.0', version: '>= 2.50'),
  dependency('gtk4'),
  dependency('libhandy-4'),
  dependency('gee-0.8'),
  dependency('sqlite3', version: '>=3.24'),
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
conf.set_quoted('APPLICATION_ID', 'dev.Cogitri.Health')
conf.set_quoted('VERSION', meson.project_version())
conf.set_quoted('DATADIR', get_option('prefix') / get_option('datadir') / 'health')

configure_file(output: 'config.h', configuration: conf)
# Include config.h
add_project_arguments(
  '-I@0@'.format(meson.current_build_dir()),
  language: 'c'
)


health_generated_sources = gnome.compile_resources('health-resources',
  'dev.Cogitri.Health.gresource.xml',
  c_name: 'health'
)

c_args = [
  '-DVERSION="@0@"'.format(meson.project_version()),
  '-DGETTEXT_PACKAGE="@0@"'.format(meson.project_name()),
  '-DGNOMELOCALEDIR="@0@"'.format(join_paths(get_option('prefix'), get_option('localedir'))),
  '-DG_LOG_DOMAIN="Health"',
]


lib = static_library(
  'dev.Cogitri.Health',
  [health_sources, health_generated_sources],
  vala_args: '--target-glib=2.50',
  dependencies: health_deps,
  c_args: c_args,
)

lib_dep = declare_dependency(
  dependencies: health_deps,
  link_with: lib,
)


executable('dev.Cogitri.Health',
  ['main.vala', health_generated_sources],
  vala_args: '--target-glib=2.50',
  dependencies: lib_dep,
  c_args: c_args,
  install: true,
)
