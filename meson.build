project('Health', 'rust',
          version: '0.94.0',
    meson_version: '>= 0.58.0',
  default_options: [ 'warning_level=2',
                   ],
)

dependency('libadwaita-1', version: '>= 1.0.1')
dependency('gtk4', version: '>= 4.6.0')
dependency('libsecret-1')
dependency('tracker-sparql-3.0', version: '>= 3.1.0')

i18n = import('i18n')
gnome = import('gnome')

version_suffix = ''
if get_option('profile') == 'development'
  profile = '.Devel'
  vcs_tag = run_command('git', 'rev-parse', '--short', 'HEAD', check: false).stdout().strip()
  if vcs_tag == ''
    version_suffix = '-devel'
  else
    version_suffix = '-@0@'.format(vcs_tag)
  endif
else
  profile = ''
endif

application_id = 'dev.Cogitri.Health@0@'.format(profile)
gettext_package = 'dev.Cogitri.Health'
health_version = meson.project_version() + version_suffix
health_data_dir = get_option('prefix') / get_option('datadir') / application_id

cargo = find_program('cargo', required: true)

subdir('data')
subdir('src')
subdir('po')

gnome.post_install(glib_compile_schemas: true, gtk_update_icon_cache: true)
meson.add_dist_script('build-aux/dist.sh', meson.project_name(), meson.project_version())

run_target('run',
  command : ['build-aux/run.sh', application_id])

run_target('potfiles-in',
  command : 'build-aux/build-potfiles-in.sh')
