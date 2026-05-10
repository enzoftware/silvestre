import { useState, useCallback } from "react";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Slider } from "@/components/ui/slider";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@/components/ui/tabs";
import { Separator } from "@/components/ui/separator";
import { FILTERS, CATEGORIES, type FilterDef, type ParamSchema } from "@/lib/filters";

interface FilterPanelProps {
  disabled: boolean;
  onApply: (filterName: string, params: Record<string, unknown>) => void;
}

export function FilterPanel({ disabled, onApply }: FilterPanelProps) {
  const [selectedFilter, setSelectedFilter] = useState<FilterDef | null>(null);
  const [paramValues, setParamValues] = useState<Record<string, number | string>>({});

  const selectFilter = useCallback((filter: FilterDef) => {
    setSelectedFilter(filter);
    const defaults: Record<string, number | string> = {};
    for (const p of filter.params) {
      defaults[p.key] = p.default;
    }
    setParamValues(defaults);
  }, []);

  const updateParam = useCallback((key: string, value: number | string) => {
    setParamValues((prev) => ({ ...prev, [key]: value }));
  }, []);

  const handleApply = useCallback(() => {
    if (!selectedFilter) return;
    onApply(selectedFilter.name, { ...paramValues });
  }, [selectedFilter, paramValues, onApply]);

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-lg">Filters</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <Tabs defaultValue="effects">
          <TabsList className="w-full">
            {CATEGORIES.map((cat) => (
              <TabsTrigger key={cat} value={cat} className="capitalize flex-1">
                {cat}
              </TabsTrigger>
            ))}
          </TabsList>
          {CATEGORIES.map((cat) => (
            <TabsContent key={cat} value={cat} className="space-y-2">
              <div className="grid grid-cols-2 gap-2">
                {FILTERS.filter((f) => f.category === cat).map((filter) => (
                  <Button
                    key={filter.name}
                    variant={selectedFilter?.name === filter.name ? "default" : "outline"}
                    size="sm"
                    onClick={() => selectFilter(filter)}
                    disabled={disabled}
                  >
                    {filter.label}
                  </Button>
                ))}
              </div>
            </TabsContent>
          ))}
        </Tabs>

        {selectedFilter && selectedFilter.params.length > 0 && (
          <>
            <Separator />
            <div className="space-y-3">
              {selectedFilter.params.map((param) => (
                <ParamControl
                  key={param.key}
                  param={param}
                  value={paramValues[param.key] ?? param.default}
                  onChange={(v) => updateParam(param.key, v)}
                  disabled={disabled}
                />
              ))}
            </div>
          </>
        )}

        {selectedFilter && (
          <>
            <Separator />
            <Button
              className="w-full"
              onClick={handleApply}
              disabled={disabled}
            >
              Apply {selectedFilter.label}
            </Button>
          </>
        )}
      </CardContent>
    </Card>
  );
}

function ParamControl({
  param,
  value,
  onChange,
  disabled,
}: {
  param: ParamSchema;
  value: number | string;
  onChange: (v: number | string) => void;
  disabled: boolean;
}) {
  if (param.type === "select" && param.options) {
    return (
      <div className="space-y-1">
        <Label>{param.label}</Label>
        <Select
          value={String(value)}
          onValueChange={(v) => onChange(v as string)}
          disabled={disabled}
        >
          <SelectTrigger>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {param.options.map((opt) => (
              <SelectItem key={opt.value} value={opt.value}>
                {opt.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
    );
  }

  const numValue = typeof value === 'number' ? value : Number(value);
  if (isNaN(numValue)) {
    return null; // or show error state
  }

  return (
    <div className="space-y-1">
      <div className="flex items-center justify-between">
        <Label>{param.label}</Label>
        <Input
          type="number"
          className="w-20 h-7 text-xs"
          value={numValue}
          min={param.min}
          max={param.max}
          step={param.step}
          onChange={(e) => {
            const n = Number(e.target.value);
            if (!isNaN(n)) onChange(n);
          }}
          disabled={disabled}
        />
      </div>
      {param.min !== undefined && param.max !== undefined && (
        <Slider
          min={param.min}
          max={param.max}
          step={param.step}
          value={numValue}
          onValueChange={(v) => onChange(v as number)}
          disabled={disabled}
        />
      )}
    </div>
  );
}
