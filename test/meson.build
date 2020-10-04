fs = import('fs')

vala_unit_dep = dependency('vala-unit', fallback: ['vala-unit', 'vala_unit_dep'])

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
  ]
)
