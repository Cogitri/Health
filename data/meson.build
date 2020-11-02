desktop_file = i18n.merge_file(
  input: 'dev.Cogitri.Health.desktop.in',
  output: 'dev.Cogitri.Health.desktop',
  type: 'desktop',
  po_dir: '../po',
  install: true,
  install_dir: join_paths(get_option('datadir'), 'applications')
)

desktop_utils = find_program('desktop-file-validate', required: false)
if desktop_utils.found()
  test('Validate desktop file', desktop_utils,
    args: [desktop_file]
  )
endif

appstream_file = i18n.merge_file(
  input: 'dev.Cogitri.Health.appdata.xml.in',
  output: 'dev.Cogitri.Health.appdata.xml',
  po_dir: '../po',
  install: true,
  install_dir: join_paths(get_option('datadir'), 'appdata')
)

appstream_util = find_program('appstream-util', required: false)
if appstream_util.found()
  test('Validate appstream file', appstream_util,
    args: ['validate-relax', appstream_file]
  )
endif

install_data('dev.Cogitri.Health.gschema.xml',
  install_dir: join_paths(get_option('datadir'), 'glib-2.0/schemas')
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
subdir('icons')
