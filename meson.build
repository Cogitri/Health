project('dev.Cogitri.Health', ['c', 'vala'], 
          version: '0.1.0',
    meson_version: '>= 0.54.0',
  default_options: [ 'warning_level=2',
                   ],
)

gnome = import('gnome')
i18n = import('i18n')
application_id = 'dev.Cogitri.Health'

subdir('data')
subdir('src')
subdir('po')
subdir('test')

meson.add_install_script('build-aux/meson/postinstall.py')

if get_option('docs')
  subdir('docs')
endif
