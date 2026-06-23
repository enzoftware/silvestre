Pod::Spec.new do |s|
  s.name             = 'silvestre_flutter'
  s.version          = '0.0.1'
  s.summary          = 'Flutter plugin for silvestre image processing library.'
  s.description      = <<-DESC
Flutter plugin for the silvestre image processing library, powered by flutter_rust_bridge.
                       DESC
  s.homepage         = 'https://github.com/enzoftware/silvestre'
  s.license          = { :file => '../LICENSE' }
  s.author           = { 'enzoftware' => 'nicoenzonz@gmail.com' }
  s.module_name      = 'silvestre_flutter'

  s.source           = { :path => '.' }
  s.source_files = 'Classes/**/*'
  s.dependency 'Flutter'
  s.platform = :ios, '11.0'

  s.pod_target_xcconfig = { 'DEFINES_MODULE' => 'YES', 'EXCLUDED_ARCHS[sdk=iphonesimulator*]' => 'i386' }
  s.swift_version = '5.0'

  s.script_phase = {
    :name => 'Build Rust library',
    :script => 'sh "$PODS_TARGET_SRCROOT/../cargokit/build_pod.sh" ../rust silvestre_flutter_rust',
    :execution_position => :before_compile,
    :input_files => ['${BUILT_PRODUCTS_DIR}/cargokit_phony'],
    :output_files => ["${PODS_CONFIGURATION_BUILD_DIR}/silvestre_flutter/libsilvestre_flutter_rust.a"],
  }
  s.pod_target_xcconfig = {
    'DEFINES_MODULE' => 'YES',
    'EXCLUDED_ARCHS[sdk=iphonesimulator*]' => 'i386',
    'OTHER_LDFLAGS' => '-force_load ${PODS_CONFIGURATION_BUILD_DIR}/silvestre_flutter/libsilvestre_flutter_rust.a',
  }
end
