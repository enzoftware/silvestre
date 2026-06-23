import 'package:flutter/material.dart';
import 'package:silvestre_flutter_example/src/filters.dart';

class FilterPanel extends StatefulWidget {
  const FilterPanel({
    required this.onApply,
    required this.isProcessing,
    super.key,
  });

  final void Function(FilterDef filter, Map<String, dynamic> params) onApply;
  final bool isProcessing;

  @override
  State<FilterPanel> createState() => _FilterPanelState();
}

class _FilterPanelState extends State<FilterPanel>
    with SingleTickerProviderStateMixin {
  late TabController _tabController;
  FilterDef? _selected;
  final Map<String, dynamic> _params = {};

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 3, vsync: this);
  }

  @override
  void dispose() {
    _tabController.dispose();
    super.dispose();
  }

  void _selectFilter(FilterDef filter) {
    setState(() {
      _selected = filter;
      _params.clear();
      for (final p in filter.params) {
        if (p.type == ParamType.slider) {
          _params[p.key] = p.defaultValue ?? p.min ?? 0;
        } else if (p.type == ParamType.select && p.options != null) {
          _params[p.key] = p.options!.first;
        }
      }
    });
  }

  List<FilterDef> _filtersFor(FilterCategory cat) =>
      filters.where((f) => f.category == cat).toList();

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        TabBar(
          controller: _tabController,
          tabs: const [
            Tab(text: 'Effects'),
            Tab(text: 'Filters'),
            Tab(text: 'Transforms'),
          ],
        ),
        SizedBox(
          height: 160,
          child: TabBarView(
            controller: _tabController,
            children: [
              _buildGrid(FilterCategory.effects),
              _buildGrid(FilterCategory.filters),
              _buildGrid(FilterCategory.transforms),
            ],
          ),
        ),
        if (_selected != null) ...[
          const SizedBox(height: 12),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: Text(_selected!.label, style: theme.textTheme.titleSmall),
          ),
          ..._buildParamControls(),
          const SizedBox(height: 12),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: FilledButton.icon(
              onPressed:
                  widget.isProcessing
                      ? null
                      : () => widget.onApply(_selected!, Map.from(_params)),
              icon:
                  widget.isProcessing
                      ? const SizedBox(
                        width: 16,
                        height: 16,
                        child: CircularProgressIndicator(strokeWidth: 2),
                      )
                      : const Icon(Icons.auto_fix_high),
              label: Text(
                widget.isProcessing
                    ? 'Processing...'
                    : 'Apply ${_selected!.label}',
              ),
            ),
          ),
          const SizedBox(height: 8),
        ],
      ],
    );
  }

  Widget _buildGrid(FilterCategory category) {
    final items = _filtersFor(category);
    return GridView.builder(
      padding: const EdgeInsets.all(12),
      gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
        crossAxisCount: 3,
        mainAxisSpacing: 8,
        crossAxisSpacing: 8,
        childAspectRatio: 2.2,
      ),
      itemCount: items.length,
      itemBuilder: (context, i) {
        final f = items[i];
        final isSelected = _selected?.name == f.name;
        return FilterChip(
          selected: isSelected,
          avatar: Icon(f.icon, size: 18),
          label: Text(
            f.label,
            style: const TextStyle(fontSize: 12),
            overflow: TextOverflow.ellipsis,
          ),
          onSelected: (_) => _selectFilter(f),
        );
      },
    );
  }

  List<Widget> _buildParamControls() {
    final controls = <Widget>[];
    for (final p in _selected!.params) {
      if (p.type == ParamType.slider) {
        final value = (_params[p.key] as num?)?.toDouble() ?? p.min ?? 0;
        controls.add(
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
            child: Row(
              children: [
                SizedBox(
                  width: 60,
                  child: Text(p.label, style: const TextStyle(fontSize: 13)),
                ),
                Expanded(
                  child: Slider(
                    value: value.clamp(p.min ?? 0, p.max ?? 100),
                    min: p.min ?? 0,
                    max: p.max ?? 100,
                    divisions:
                        p.step != null
                            ? ((p.max! - p.min!) / p.step!).round()
                            : null,
                    label: _formatValue(value, p),
                    onChanged: (v) {
                      setState(() {
                        if (p.step != null && p.step! >= 1) {
                          _params[p.key] = v.round();
                        } else {
                          _params[p.key] = double.parse(v.toStringAsFixed(1));
                        }
                      });
                    },
                  ),
                ),
                SizedBox(
                  width: 48,
                  child: Text(
                    _formatValue(value, p),
                    style: const TextStyle(fontSize: 12),
                    textAlign: TextAlign.end,
                  ),
                ),
              ],
            ),
          ),
        );
      } else if (p.type == ParamType.select && p.options != null) {
        final value = _params[p.key] as String? ?? p.options!.first;
        controls.add(
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
            child: Row(
              children: [
                SizedBox(
                  width: 60,
                  child: Text(p.label, style: const TextStyle(fontSize: 13)),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: SegmentedButton<String>(
                    segments:
                        p.options!
                            .map(
                              (o) => ButtonSegment(
                                value: o,
                                label: Text(
                                  o[0].toUpperCase() + o.substring(1),
                                  style: const TextStyle(fontSize: 12),
                                ),
                              ),
                            )
                            .toList(),
                    selected: {value},
                    onSelectionChanged: (v) {
                      setState(() => _params[p.key] = v.first);
                    },
                  ),
                ),
              ],
            ),
          ),
        );
      }
    }
    return controls;
  }

  String _formatValue(double value, FilterParam p) {
    if (p.step != null && p.step! >= 1) return value.round().toString();
    return value.toStringAsFixed(1);
  }
}
