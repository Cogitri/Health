scalable_dir = 'hicolor' / 'scalable' / 'apps'

install_data(
  scalable_dir / '@0@.svg'.format(application_id),
  install_dir: get_option('datadir') / 'icons' / scalable_dir,
  rename: '@0@.svg'.format(application_id)
)

symbolic_dir = 'hicolor' / 'symbolic' / 'apps'

install_data(
  symbolic_dir / 'dev.Cogitri.Health-symbolic.svg',
  install_dir: get_option('datadir') / 'icons' / symbolic_dir,
  rename: '@0@-symbolic.svg'.format(application_id)
)

