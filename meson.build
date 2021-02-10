project('Health',  
          version: '0.91.0',
    meson_version: '>= 0.50.0',
  default_options: [ 'warning_level=2',
                   ],
)

i18n = import('i18n')
gnome = import('gnome')

version_suffix = ''
if get_option('debug')
  profile = '.Devel'
  vcs_tag = run_command('git', 'rev-parse', '--short', 'HEAD').stdout().strip()
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

subdir('data')
subdir('src')
subdir('po')

meson.add_install_script('build-aux/meson/postinstall.py')
