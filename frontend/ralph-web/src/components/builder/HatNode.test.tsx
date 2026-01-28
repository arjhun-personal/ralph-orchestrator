/**
 * HatNode Component Tests
 *
 * Tests that HatNode correctly renders hat metadata,
 * connection handles for triggers/publishes, and
 * selected state styling.
 */

import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";

// Mock @xyflow/react since it uses DOM APIs not available in jsdom
vi.mock("@xyflow/react", () => ({
  Handle: ({ id, type, position, ...props }: any) => (
    <div data-testid={`handle-${type}-${id}`} {...props} />
  ),
  Position: { Left: "left", Right: "right" },
}));

import { HatNode, type HatNodeData } from "./HatNode";

function makeHatData(overrides: Partial<HatNodeData> = {}): HatNodeData {
  return {
    key: "builder",
    name: "Builder",
    description: "Implements code, runs tests, creates commits",
    triggersOn: ["build.task"],
    publishes: ["build.done", "build.blocked"],
    ...overrides,
  };
}

describe("HatNode", () => {
  it("renders hat name and description", () => {
    const data = makeHatData({
      name: "Planner",
      description: "Analyzes tasks and creates plans",
    });

    render(<HatNode data={data} />);

    expect(screen.getByText("Planner")).toBeInTheDocument();
    expect(screen.getByText("Analyzes tasks and creates plans")).toBeInTheDocument();
  });

  it("shows correct trigger and publish counts", () => {
    const data = makeHatData({
      triggersOn: ["work.start", "build.blocked"],
      publishes: ["build.task"],
    });

    render(<HatNode data={data} />);

    expect(screen.getByText("2 triggers")).toBeInTheDocument();
    expect(screen.getByText("1 publish")).toBeInTheDocument();
  });

  it("shows default handles when no triggers or publishes are defined", () => {
    const data = makeHatData({
      triggersOn: [],
      publishes: [],
    });

    render(<HatNode data={data} />);

    expect(screen.getByTestId("handle-target-default-in")).toBeInTheDocument();
    expect(screen.getByTestId("handle-source-default-out")).toBeInTheDocument();
  });

  it("renders with selected styling", () => {
    const data = makeHatData();

    const { container } = render(<HatNode data={data} selected={true} />);

    const outerDiv = container.firstElementChild as HTMLElement;
    expect(outerDiv.className).toContain("ring-2");
  });

  it("renders without selected styling by default", () => {
    const data = makeHatData();

    const { container } = render(<HatNode data={data} />);

    const outerDiv = container.firstElementChild as HTMLElement;
    expect(outerDiv.className).not.toContain("ring-2");
  });
});
