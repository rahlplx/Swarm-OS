import { useState } from "react";

interface ResourceSliderProps {
  label: string;
  min?: number;
  max?: number;
  defaultValue?: number;
  onChange?: (value: number) => void;
}

export function ResourceSlider({
  label,
  min = 0,
  max = 100,
  defaultValue = 50,
  onChange,
}: ResourceSliderProps) {
  const [value, setValue] = useState(defaultValue);

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const v = parseInt(e.target.value, 10);
    setValue(v);
    onChange?.(v);
  };

  return (
    <div data-testid="resource-slider">
      <label>
        {label}: {value}%
        <input
          type="range"
          min={min}
          max={max}
          value={value}
          onChange={handleChange}
          data-testid="slider-input"
        />
      </label>
    </div>
  );
}
