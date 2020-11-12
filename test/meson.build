fs = import('fs')

vala_unit_proj = subproject('vala-unit', default_options: ['install=disabled', 'valadoc=disabled'])

vala_unit_dep = vala_unit_proj.get_variable('vala_unit_dep')

test_sources = [
  'database.vala',
  'test_main.vala',
  'units.vala',
]

test_exe = executable(
  'test-main',
  test_sources,
  dependencies: [lib_dep, vala_unit_dep],
)

test(
  'test-main',
  test_exe,
  env: [
    'GSETTINGS_SCHEMA_DIR=' + meson.current_build_dir() / '..' / 'data',
    'GSETTINGS_BACKEND=memory',
    'HEALTH_ONTOLOGY_OVERRIDE_PATH=' + meson.current_source_dir() / '..' / 'data' / 'tracker' / 'ontology',
  ]
)
