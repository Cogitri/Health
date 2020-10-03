test_sources = [
  'units.vala',
]

fs = import('fs')

foreach src : test_sources
    lib_test_exe = executable(
        'test_' + fs.stem(src) + '_exe',
        src,
        dependencies: lib_dep,
    )
    test(fs.stem(src), lib_test_exe)
endforeach
