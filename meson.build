project(
  'gnome-stocks',
  'c',
  meson_version : '>= 1.3.0',
  version : '0.1',
  default_options : ['warning_level=3'],
)

dependencies = [
  dependency('gtk4'),
  dependency('libadwaita-1'),
  dependency('libcurl'),
  dependency('cJSON'),
]

sources = [
  'src/api/curl.c',
  'src/api/yahoo.c',
  'src/application.c',
  'src/main.c',
  'src/model/price.c',
  'src/model/stock.c',
  'src/settings.c',
  'src/ui/sidebar.c',
  'src/ui/stock_view.c',
  'src/ui/window.c',
]

schemas_dir = get_option('prefix') / get_option('datadir') / 'glib-2.0' / 'schemas'
settings_schemas = [ 'org.gnome.Stocks.gschema.xml' ]

install_data(settings_schemas, install_dir: schemas_dir)

gnome = import('gnome')
gnome.compile_schemas(depend_files: files(settings_schemas))

meson.add_install_script('glib-compile-schemas', schemas_dir)

exe = executable(
  'gnome-stocks',
  sources,
  dependencies : dependencies,
  install : true,
)

test('basic', exe)




