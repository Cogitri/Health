conf = configuration_data()
conf.set_quoted('VERSION', meson.project_version())
conf.set_quoted('localedir', get_option('prefix') / get_option('localedir'))
conf.set_quoted('pkgdatadir', health_data_dir)
conf.set_quoted('application_id', application_id)
conf.set_quoted('log_domain', application_id)
conf.set_quoted('gettext_package', gettext_package)
conf.set_quoted('autostart_desktop_file_path', get_option('prefix') / get_option('datadir') / 'applications' / 'dev.Cogitri.Health.Autostart.desktop')
conf.set_quoted('blueprint_compiler_path', find_program('blueprint-compiler').full_path())

configure_file(
    input: 'config.rs.in',
    output: 'config.rs',
    configuration: conf
)

# Copy the config.rs output to the source directory.
run_command(
  'cp',
  meson.project_build_root() / 'src' / 'config.rs',
  meson.project_source_root() / 'src' / 'config.rs',
  check: true
)

cargo_options = [ '--manifest-path', meson.project_source_root() / 'Cargo.toml' ]
cargo_options += [ '--target-dir', meson.project_build_root() / 'src' ]

if get_option('profile') == 'default'
  cargo_options += [ '--release' ]
  rust_target = 'release'
  message('Building in release mode')
else
  rust_target = 'debug'
  message('Building in debug mode')
endif

cargo_env = [ 'CARGO_HOME=' + meson.build_root() / 'cargo-home' ]

cargo_build = custom_target(
  'cargo-build',
  build_by_default: true,
  build_always_stale: true,
  output: application_id,
  console: true,
  install: true,
  install_dir: get_option('bindir'),
  command: [
    'env',
    cargo_env,
    cargo, 'build',
    cargo_options,
    '&&',
    'cp', 'src' / rust_target / 'health', '@OUTPUT@',
  ]
)

test(
  'cargo-tests',
  find_program(meson.project_source_root() / 'build-aux' / 'cargo-test.sh'),
  args: [
    meson.current_build_dir() / '..',
    meson.project_source_root(),
    get_option('buildtype'),
  ],
  timeout: 3000, # might take a bit longer due to cargo...
)
