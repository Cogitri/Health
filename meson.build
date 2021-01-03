project('dev.Cogitri.Health', ['c', 'vala'], 
          version: '0.90.1',
    meson_version: '>= 0.54.0',
  default_options: [ 'warning_level=2',
                   ],
)

gnome = import('gnome')
i18n = import('i18n')

version_suffix = ''
if get_option('profile') == 'development'
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
health_version = meson.project_version() + version_suffix
health_data_dir = get_option('prefix') / get_option('datadir') / application_id

subdir('data')
subdir('src')
subdir('po')
subdir('test')

meson.add_install_script('build-aux/meson/postinstall.py')

if get_option('docs')
  subdir('docs')
endif
