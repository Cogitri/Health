conf = configuration_data ()
conf.set('APPLICATION_ID', application_id)
conf.set('GETTEXT_PACKAGE', gettext_package)
conf.set('BIN_DIR', get_option('prefix') / get_option('bindir'))

configured_desktop_file = configure_file(
  input: 'dev.Cogitri.Health.desktop.in.in',
  output: '@BASENAME@',
  configuration: conf,
)

desktop_file = i18n.merge_file(
  input: configured_desktop_file,
  output: '@0@.desktop'.format(application_id),
  type: 'desktop',
  po_dir: '../po',
  install: true,
  install_dir: get_option('datadir') / 'applications'
)

configured_autostart_file = configure_file(
  input: 'dev.Cogitri.Health.Autostart.desktop.in.in',
  output: '@BASENAME@',
  configuration: conf,
)

autostart_desktop_file = i18n.merge_file(
  input: configured_autostart_file,
  output: '@BASENAME@',
  type: 'desktop',
  po_dir: '../po',
  install: true,
  install_dir: get_option('datadir') / 'applications',
)

desktop_utils = find_program('desktop-file-validate', required: false)
if desktop_utils.found()
  test('Validate desktop file', desktop_utils,
    args: [autostart_desktop_file, desktop_file]
  )
endif

configured_appstream_file = configure_file(
  input: 'dev.Cogitri.Health.metainfo.xml.in.in',
  output: '@BASENAME@',
  configuration: conf,
)

appstream_file = i18n.merge_file(
  input: configured_appstream_file,
  output: '@0@.metainfo.xml'.format(application_id),
  po_dir: '../po',
  install: true,
  install_dir: get_option('datadir') / 'metainfo',
)

appstream_util = find_program('appstream-util', required: false)
if appstream_util.found()
  test('Validate appstream file', appstream_util,
    args: ['validate-relax', appstream_file]
  )
endif

install_data(
  'dev.Cogitri.Health.gschema.xml',
  rename: '@0@.gschema.xml'.format(application_id),
  install_dir: get_option('datadir') / 'glib-2.0' / 'schemas',
)

compile_schemas = find_program('glib-compile-schemas', required: false)
if compile_schemas.found()
  test('Validate schema file', compile_schemas,
    args: ['--strict', '--dry-run', meson.current_source_dir()]
  )
endif

gnome.compile_schemas(
  build_by_default: true,
  depend_files: 'dev.Cogitri.Health.gschema.xml',
)

blueprints = custom_target('blueprints',
  input: files(
    'ui/activity_type_row.blp',
    'ui/bmi_level_bar.blp',
    'ui/sync_list_box.blp',
    'ui/tab_button.blp',
    'ui/view_add_activity.blp',
    'ui/view_add_weight.blp',
    'ui/data_add_dialog.blp',
    'ui/date_selector.blp',
    'ui/import_export_dialog_base.blp',
    'ui/plugins/activities/details.blp',
    'ui/plugins/activities/summary.blp',
    'ui/plugins/calories/details.blp',
    'ui/plugins/calories/summary.blp',
    'ui/plugins/steps/details.blp',
    'ui/plugins/steps/summary.blp',
    'ui/plugins/weight/details.blp',
    'ui/plugins/weight/summary.blp',
    'ui/plugins/overview.blp',
    'ui/plugins/details.blp',
    'ui/activity_type_selector.blp',
    'ui/card.blp',
    'ui/distance_action_row.blp',
    'ui/legend_row.blp',
    'ui/preferences_window.blp',
    'ui/setup_window.blp',
    'ui/shortcuts_window.blp',
    'ui/view_home_page.blp',
    'ui/window.blp',
    'ui/password_entry.blp',
  ),
  output: '.',
  command: [find_program('blueprint-compiler'), 'batch-compile', '@OUTPUT@', '@CURRENT_SOURCE_DIR@', '@INPUT@'],
)

gnome.compile_resources('dev.Cogitri.Health',
  'dev.Cogitri.Health.gresource.xml',
  dependencies: blueprints,
  gresource_bundle: true,
  install: true,
  install_dir: health_data_dir,
)

configure_file(
  input: 'dev.Cogitri.Health.service.in',
  output: '@0@.service'.format(application_id),
  configuration: conf,
  install: true,
  install_dir: get_option('datadir') / 'dbus-1' / 'services',
)

subdir('icons')
subdir('tracker')
