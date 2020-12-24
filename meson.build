project('dev.Cogitri.Health', ['c', 'vala'], 
          version: '0.1.0',
    meson_version: '>= 0.54.0',
  default_options: [ 'warning_level=2',
                   ],
)

gnome = import('gnome')
i18n = import('i18n')
if get_option('profile') == 'development'
  profile = '.Devel'
else
  profile = ''
endif

application_id = 'dev.Cogitri.Health@0@'.format(profile)
health_data_dir = get_option('prefix') / get_option('datadir') / application_id

subdir('data')
subdir('src')
subdir('po')
subdir('test')

meson.add_install_script('build-aux/meson/postinstall.py')

if get_option('docs')
  subdir('docs')
endif
