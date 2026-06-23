import { render, screen } from "../../lib/test-utils";
import { fireEvent } from "@testing-library/react";
import { ResourceSlider } from "./ResourceSlider";

describe("ResourceSlider", () => {
  it("renders with default value", () => {
    render(<ResourceSlider label="GPU Donation" />);
    expect(screen.getByTestId("resource-slider")).toHaveTextContent("GPU Donation: 50%");
  });

  it("renders with custom default", () => {
    render(<ResourceSlider label="CPU" defaultValue={75} />);
    expect(screen.getByTestId("resource-slider")).toHaveTextContent("CPU: 75%");
  });

  it("calls onChange when slider moves", () => {
    const onChange = vi.fn();
    render(<ResourceSlider label="GPU" onChange={onChange} />);

    const slider = screen.getByTestId("slider-input");
    fireEvent.change(slider, { target: { value: "80" } });

    expect(onChange).toHaveBeenCalledWith(80);
  });
});
