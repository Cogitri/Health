fs = import('fs')

vala_unit_dep = dependency('vala-unit', fallback: ['vala-unit', 'vala_unit_dep'])

test_sources = [
  'test_main.vala',
  'test_settings.vala',
  'units.vala',
]

test_exe = executable(
  'test-main',
  test_sources,
  dependencies: [lib_dep, vala_unit_dep],
)

test('test-main', test_exe)
