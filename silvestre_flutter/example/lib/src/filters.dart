import 'package:flutter/material.dart';

enum FilterCategory { effects, filters, transforms }

enum ParamType { slider, select }

class FilterParam {
  const FilterParam({
    required this.key,
    required this.label,
    required this.type,
    this.min,
    this.max,
    this.step,
    this.defaultValue,
    this.options,
  });

  final String key;
  final String label;
  final ParamType type;
  final double? min;
  final double? max;
  final double? step;
  final double? defaultValue;
  final List<String>? options;
}

class FilterDef {
  const FilterDef({
    required this.name,
    required this.label,
    required this.icon,
    required this.category,
    this.params = const [],
  });

  final String name;
  final String label;
  final IconData icon;
  final FilterCategory category;
  final List<FilterParam> params;
}

const filters = <FilterDef>[
  // Effects
  FilterDef(
    name: 'grayscale',
    label: 'Grayscale',
    icon: Icons.filter_b_and_w,
    category: FilterCategory.effects,
  ),
  FilterDef(
    name: 'invert',
    label: 'Invert',
    icon: Icons.invert_colors,
    category: FilterCategory.effects,
  ),
  FilterDef(
    name: 'sepia',
    label: 'Sepia',
    icon: Icons.filter_vintage,
    category: FilterCategory.effects,
  ),
  FilterDef(
    name: 'brightness',
    label: 'Brightness',
    icon: Icons.brightness_6,
    category: FilterCategory.effects,
    params: [
      FilterParam(
        key: 'delta',
        label: 'Delta',
        type: ParamType.slider,
        min: -255,
        max: 255,
        step: 1,
        defaultValue: 30,
      ),
    ],
  ),
  FilterDef(
    name: 'contrast',
    label: 'Contrast',
    icon: Icons.contrast,
    category: FilterCategory.effects,
    params: [
      FilterParam(
        key: 'factor',
        label: 'Factor',
        type: ParamType.slider,
        min: 0.1,
        max: 3,
        step: 0.1,
        defaultValue: 1.5,
      ),
    ],
  ),

  // Filters
  FilterDef(
    name: 'sharpen',
    label: 'Sharpen',
    icon: Icons.deblur,
    category: FilterCategory.filters,
  ),
  FilterDef(
    name: 'box_blur',
    label: 'Box Blur',
    icon: Icons.blur_on,
    category: FilterCategory.filters,
  ),
  FilterDef(
    name: 'sobel',
    label: 'Sobel Edge',
    icon: Icons.border_style,
    category: FilterCategory.filters,
  ),
  FilterDef(
    name: 'gaussian',
    label: 'Gaussian',
    icon: Icons.blur_circular,
    category: FilterCategory.filters,
    params: [
      FilterParam(
        key: 'sigma',
        label: 'Sigma',
        type: ParamType.slider,
        min: 0.1,
        max: 10,
        step: 0.1,
        defaultValue: 2,
      ),
    ],
  ),
  FilterDef(
    name: 'median',
    label: 'Median',
    icon: Icons.grain,
    category: FilterCategory.filters,
    params: [
      FilterParam(
        key: 'size',
        label: 'Size',
        type: ParamType.slider,
        min: 3,
        max: 15,
        step: 2,
        defaultValue: 3,
      ),
    ],
  ),
  FilterDef(
    name: 'canny',
    label: 'Canny Edge',
    icon: Icons.auto_graph,
    category: FilterCategory.filters,
    params: [
      FilterParam(
        key: 'low',
        label: 'Low',
        type: ParamType.slider,
        min: 0,
        max: 100,
        step: 1,
        defaultValue: 50,
      ),
      FilterParam(
        key: 'high',
        label: 'High',
        type: ParamType.slider,
        min: 0,
        max: 200,
        step: 1,
        defaultValue: 100,
      ),
      FilterParam(
        key: 'sigma',
        label: 'Sigma',
        type: ParamType.slider,
        min: 0.1,
        max: 5,
        step: 0.1,
        defaultValue: 1.4,
      ),
    ],
  ),

  // Transforms
  FilterDef(
    name: 'crop',
    label: 'Crop',
    icon: Icons.crop,
    category: FilterCategory.transforms,
    params: [
      FilterParam(
        key: 'x',
        label: 'X',
        type: ParamType.slider,
        min: 0,
        max: 4096,
        step: 1,
        defaultValue: 0,
      ),
      FilterParam(
        key: 'y',
        label: 'Y',
        type: ParamType.slider,
        min: 0,
        max: 4096,
        step: 1,
        defaultValue: 0,
      ),
      FilterParam(
        key: 'w',
        label: 'Width',
        type: ParamType.slider,
        min: 1,
        max: 4096,
        step: 1,
        defaultValue: 256,
      ),
      FilterParam(
        key: 'h',
        label: 'Height',
        type: ParamType.slider,
        min: 1,
        max: 4096,
        step: 1,
        defaultValue: 256,
      ),
    ],
  ),
  FilterDef(
    name: 'resize',
    label: 'Resize',
    icon: Icons.photo_size_select_large,
    category: FilterCategory.transforms,
    params: [
      FilterParam(
        key: 'w',
        label: 'Width',
        type: ParamType.slider,
        min: 1,
        max: 4096,
        step: 1,
        defaultValue: 512,
      ),
      FilterParam(
        key: 'h',
        label: 'Height',
        type: ParamType.slider,
        min: 1,
        max: 4096,
        step: 1,
        defaultValue: 512,
      ),
    ],
  ),
  FilterDef(
    name: 'rotate',
    label: 'Rotate',
    icon: Icons.rotate_right,
    category: FilterCategory.transforms,
    params: [
      FilterParam(
        key: 'angle',
        label: 'Angle',
        type: ParamType.slider,
        min: -360,
        max: 360,
        step: 1,
        defaultValue: 90,
      ),
    ],
  ),
  FilterDef(
    name: 'mirror',
    label: 'Mirror',
    icon: Icons.flip,
    category: FilterCategory.transforms,
    params: [
      FilterParam(
        key: 'mode',
        label: 'Mode',
        type: ParamType.select,
        options: ['horizontal', 'vertical', 'both'],
      ),
    ],
  ),
];
