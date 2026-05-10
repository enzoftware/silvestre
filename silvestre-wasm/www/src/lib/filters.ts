export type ParamType = "int" | "float" | "select";

export interface ParamSchema {
  key: string;
  label: string;
  type: ParamType;
  min?: number;
  max?: number;
  step?: number;
  default: number | string;
  options?: { label: string; value: string }[];
}

export interface FilterDef {
  name: string;
  label: string;
  category: "effects" | "filters" | "transforms";
  params: ParamSchema[];
}

export const FILTERS: FilterDef[] = [
  // Effects — no params
  { name: "grayscale", label: "Grayscale", category: "effects", params: [] },
  { name: "invert", label: "Invert", category: "effects", params: [] },
  { name: "sepia", label: "Sepia", category: "effects", params: [] },

  // Effects — with params
  {
    name: "brightness",
    label: "Brightness",
    category: "effects",
    params: [
      { key: "delta", label: "Delta", type: "int", min: -255, max: 255, step: 1, default: 30 },
    ],
  },
  {
    name: "contrast",
    label: "Contrast",
    category: "effects",
    params: [
      { key: "factor", label: "Factor", type: "float", min: 0.1, max: 3.0, step: 0.1, default: 1.5 },
    ],
  },

  // Filters — no params
  { name: "sharpen", label: "Sharpen", category: "filters", params: [] },
  { name: "box_blur", label: "Box Blur", category: "filters", params: [] },
  { name: "sobel", label: "Sobel Edge", category: "filters", params: [] },

  // Filters — with params
  {
    name: "gaussian",
    label: "Gaussian Blur",
    category: "filters",
    params: [
      { key: "sigma", label: "Sigma", type: "float", min: 0.1, max: 10.0, step: 0.1, default: 2.0 },
    ],
  },
  {
    name: "median",
    label: "Median",
    category: "filters",
    params: [
      { key: "size", label: "Size", type: "int", min: 3, max: 15, step: 2, default: 3 },
    ],
  },
  {
    name: "canny",
    label: "Canny Edge",
    category: "filters",
    params: [
      { key: "low", label: "Low threshold", type: "float", min: 0.0, max: 100.0, step: 1.0, default: 50.0 },
      { key: "high", label: "High threshold", type: "float", min: 0.0, max: 200.0, step: 1.0, default: 100.0 },
      { key: "sigma", label: "Sigma", type: "float", min: 0.1, max: 5.0, step: 0.1, default: 1.4 },
    ],
  },

  // Transforms
  {
    name: "resize",
    label: "Resize",
    category: "transforms",
    params: [
      { key: "w", label: "Width", type: "int", min: 1, max: 4096, step: 1, default: 512 },
      { key: "h", label: "Height", type: "int", min: 1, max: 4096, step: 1, default: 512 },
    ],
  },
  {
    name: "rotate",
    label: "Rotate",
    category: "transforms",
    params: [
      { key: "angle", label: "Angle (deg)", type: "float", min: -360, max: 360, step: 1, default: 90 },
    ],
  },
  {
    name: "mirror",
    label: "Mirror",
    category: "transforms",
    params: [
      {
        key: "mode",
        label: "Mode",
        type: "select",
        default: "horizontal",
        options: [
          { label: "Horizontal", value: "horizontal" },
          { label: "Vertical", value: "vertical" },
          { label: "Both", value: "both" },
        ],
      },
    ],
  },
  {
    name: "crop",
    label: "Crop",
    category: "transforms",
    params: [
      { key: "x", label: "X", type: "int", min: 0, max: 4096, step: 1, default: 0 },
      { key: "y", label: "Y", type: "int", min: 0, max: 4096, step: 1, default: 0 },
      { key: "w", label: "Width", type: "int", min: 1, max: 4096, step: 1, default: 256 },
      { key: "h", label: "Height", type: "int", min: 1, max: 4096, step: 1, default: 256 },
    ],
  },
];

export const CATEGORIES = ["effects", "filters", "transforms"] as const;
