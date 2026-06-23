import { render, screen } from "../../lib/test-utils";
import { DownloadProgress } from "./DownloadProgress";

describe("DownloadProgress", () => {
  it("renders progress bar with correct percentage", () => {
    render(<DownloadProgress modelName="llama-3-8b" progress={45.5} totalBytes={4_500_000_000} />);

    expect(screen.getByTestId("model-name")).toHaveTextContent("llama-3-8b");
    expect(screen.getByTestId("progress-text")).toHaveTextContent("45.5%");
  });

  it("shows 0% at start", () => {
    render(<DownloadProgress modelName="test" progress={0} totalBytes={1_000_000_000} />);

    expect(screen.getByTestId("progress-text")).toHaveTextContent("0.0%");
  });

  it("shows 100% when complete", () => {
    render(<DownloadProgress modelName="test" progress={100} totalBytes={2_000_000_000} />);

    expect(screen.getByTestId("progress-text")).toHaveTextContent("100.0%");
    expect(screen.getByTestId("progress-text")).toHaveTextContent("2.0 / 2.0 GB");
  });
});
