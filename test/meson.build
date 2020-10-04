fs = import('fs')

test_sources = [
  'units.vala',
]

valaunit_dep = dependency('vala-unit', fallback: ['vala-unit', 'vala_unit_dep'])

foreach src : test_sources
    lib_test_exe = executable(
        'test_' + fs.stem(src) + '_exe',
        src,
        dependencies: lib_dep,
    )
    test(fs.stem(src), lib_test_exe)
endforeach
