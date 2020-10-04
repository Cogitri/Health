fs = import('fs')

test_lib_sources = [
  'test_settings.vala'
]

test_lib = static_library(
  'test-lib',
  test_lib_sources,
  dependencies: lib_dep,
)

test_lib_dep = declare_dependency(
  link_with: test_lib,
  dependencies: lib_dep,
)

test_sources = [
  'units.vala',
]

valaunit_dep = dependency('vala-unit', fallback: ['vala-unit', 'vala_unit_dep'])

foreach src : test_sources
    lib_test_exe = executable(
        'test_' + fs.stem(src) + '_exe',
        src,
        dependencies: test_lib_dep,
    )
    test(fs.stem(src), lib_test_exe)
endforeach
