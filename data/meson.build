gnome = import('gnome')

# Install desktop file
desktop_file = configure_file(
  input: 'io.github.didley.CamOverlay.desktop.in',
  output: 'io.github.didley.CamOverlay.desktop',
  configuration: {'APP_ID': app_id},
)
install_data(desktop_file,
  install_dir: datadir / 'applications',
)

# Install metainfo
install_data('io.github.didley.CamOverlay.metainfo.xml',
  install_dir: datadir / 'metainfo',
)

# Compile and install GSettings schema
install_data('io.github.didley.CamOverlay.gschema.xml',
  install_dir: datadir / 'glib-2.0' / 'schemas',
)
gnome.compile_schemas(
  depend_files: 'io.github.didley.CamOverlay.gschema.xml',
)

# Install CSS
install_data('style.css',
  install_dir: datadir / app_id,
)

# Install icon
install_data(
  'icons/hicolor/scalable/apps/io.github.didley.CamOverlay.svg',
  install_dir: datadir / 'icons' / 'hicolor' / 'scalable' / 'apps',
)
