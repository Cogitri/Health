gnome_health_sources = [
  'add_dialog.vala',
  'application.vala',
  'database.vala',
  'main.vala',
  'primary_menu.vala',
  'steps.vala',
  'util.vala',
  'views.vala',
  'weight.vala',
  'window.vala',
]

gnome_health_deps = [
  dependency('gio-2.0', version: '>= 2.50'),
  dependency('gtk+-3.0', version: '>= 3.24'),
  dependency('libhandy-1'),
  dependency('caroline-0.2'),
  dependency('gee-0.8'),
  dependency('sqlite3', version: '>=3.24'),
]

gnome = import('gnome')

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
conf.set_quoted('APPLICATION_ID', 'org.gnome.Health')
conf.set_quoted('VERSION', meson.project_version())

configure_file(output: 'config.h', configuration: conf)
# Include config.h
add_project_arguments(
  '-I@0@'.format(meson.current_build_dir()),
  language: 'c'
)


gnome_health_sources += gnome.compile_resources('gnome_health-resources',
  'gnome_health.gresource.xml',
  c_name: 'gnome_health'
)

c_args = [
  '-DVERSION="@0@"'.format(meson.project_version()),
  '-DGETTEXT_PACKAGE="@0@"'.format(meson.project_name()),
  '-DGNOMELOCALEDIR="@0@"'.format(join_paths(get_option('prefix'), get_option('localedir'))),
]



executable('gnome-health', gnome_health_sources,
  vala_args: '--target-glib=2.50',  dependencies: gnome_health_deps,
  install: true,
  c_args: c_args,
)
